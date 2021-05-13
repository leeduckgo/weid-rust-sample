use crate::config;

use crate::infer_schema_internals::*;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_regex::Serde as RegexWrapper;
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write};
use std::io::Write as IoWrite;

const SCHEMA_HEADER: &str = "// @generated automatically by Diesel CLI.\n";

type Regex = RegexWrapper<::regex::Regex>;

pub enum Filtering {
    OnlyTables(Vec<Regex>),
    ExceptTables(Vec<Regex>),
    None,
}

impl Default for Filtering {
    fn default() -> Self {
        Filtering::None
    }
}

impl Filtering {
    pub fn should_ignore_table(&self, name: &TableName) -> bool {
        use self::Filtering::*;

        match *self {
            OnlyTables(ref regexes) => !regexes.iter().any(|regex| regex.is_match(&name.sql_name)),
            ExceptTables(ref regexes) => regexes.iter().any(|regex| regex.is_match(&name.sql_name)),
            None => false,
        }
    }
}

/// How to sort columns when querying the table schema.
#[derive(Debug, Deserialize, Serialize)]
pub enum ColumnSorting {
    /// Order by ordinal position
    #[serde(rename = "ordinal_position")]
    OrdinalPosition,
    /// Order by column name
    #[serde(rename = "name")]
    Name,
}

impl Default for ColumnSorting {
    fn default() -> Self {
        ColumnSorting::OrdinalPosition
    }
}

pub fn run_print_schema<W: IoWrite>(
    database_url: &str,
    config: &config::PrintSchema,
    output: &mut W,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let schema = output_schema(database_url, config)?;

    output.write_all(schema.as_bytes())?;
    Ok(())
}

pub fn output_schema(
    database_url: &str,
    config: &config::PrintSchema,
) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
    let table_names = load_table_names(database_url, config.schema_name())?
        .into_iter()
        .filter(|t| !config.filter.should_ignore_table(t))
        .collect::<Vec<_>>();
    let foreign_keys = load_foreign_key_constraints(database_url, config.schema_name())?;
    let foreign_keys =
        remove_unsafe_foreign_keys_for_codegen(database_url, &foreign_keys, &table_names);
    let table_data = table_names
        .into_iter()
        .map(|t| load_table_data(database_url, t, &config.column_sorting))
        .collect::<Result<_, Box<dyn Error + Send + Sync + 'static>>>()?;
    let definitions = TableDefinitions {
        tables: table_data,
        fk_constraints: foreign_keys,
        include_docs: config.with_docs,
        import_types: config.import_types(),
    };

    let mut out = String::new();
    writeln!(out, "{}", SCHEMA_HEADER)?;

    if let Some(schema_name) = config.schema_name() {
        write!(out, "{}", ModuleDefinition(schema_name, definitions))?;
    } else {
        write!(out, "{}", definitions)?;
    }

    if let Some(ref patch_file) = config.patch_file {
        let patch = std::fs::read_to_string(patch_file)?;
        let patch = diffy::Patch::from_str(&patch)?;

        out = diffy::apply(&out, &patch)?;
    }

    Ok(out)
}

struct ModuleDefinition<'a>(&'a str, TableDefinitions<'a>);

impl<'a> Display for ModuleDefinition<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        {
            let mut out = PadAdapter::new(f);
            writeln!(out, "pub mod {} {{", self.0)?;
            write!(out, "{}", self.1)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

struct TableDefinitions<'a> {
    tables: Vec<TableData>,
    fk_constraints: Vec<ForeignKeyConstraint>,
    include_docs: bool,
    import_types: Option<&'a [String]>,
}

impl<'a> Display for TableDefinitions<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut is_first = true;
        for table in &self.tables {
            if is_first {
                is_first = false;
            } else {
                writeln!(f)?;
            }
            writeln!(
                f,
                "{}",
                TableDefinition {
                    table,
                    include_docs: self.include_docs,
                    import_types: self.import_types,
                }
            )?;
        }

        if !self.fk_constraints.is_empty() {
            writeln!(f)?;
        }

        for foreign_key in &self.fk_constraints {
            writeln!(f, "{}", Joinable(foreign_key))?;
        }

        if self.tables.len() > 1 {
            write!(f, "\ndiesel::allow_tables_to_appear_in_same_query!(")?;
            {
                let mut out = PadAdapter::new(f);
                writeln!(out)?;
                for table in &self.tables {
                    if table.name.rust_name == table.name.sql_name {
                        writeln!(out, "{},", table.name.sql_name)?;
                    } else {
                        writeln!(out, "{},", table.name.rust_name)?;
                    }
                }
            }
            writeln!(f, ");")?;
        }

        Ok(())
    }
}

struct TableDefinition<'a> {
    table: &'a TableData,
    import_types: Option<&'a [String]>,
    include_docs: bool,
}

impl<'a> Display for TableDefinition<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "diesel::table! {{")?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;

            if let Some(types) = self.import_types {
                for import in types {
                    writeln!(out, "use {};", import)?;
                }
                writeln!(out)?;
            }

            if self.include_docs {
                for d in self.table.docs.lines() {
                    writeln!(out, "///{}{}", if d.is_empty() { "" } else { " " }, d)?;
                }
            }

            if self.table.name.rust_name != self.table.name.sql_name {
                writeln!(
                    out,
                    r#"#[sql_name = "{}"]"#,
                    self.table.name.full_sql_name()
                )?;
            }

            write!(out, "{} (", self.table.name)?;

            for (i, pk) in self.table.primary_key.iter().enumerate() {
                if i != 0 {
                    write!(out, ", ")?;
                }
                write!(out, "{}", pk)?;
            }

            write!(
                out,
                ") {}",
                ColumnDefinitions {
                    columns: &self.table.column_data,
                    include_docs: self.include_docs,
                }
            )?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

struct ColumnDefinitions<'a> {
    columns: &'a [ColumnDefinition],
    include_docs: bool,
}

impl<'a> Display for ColumnDefinitions<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        {
            let mut out = PadAdapter::new(f);
            writeln!(out, "{{")?;
            for column in self.columns {
                if self.include_docs {
                    for d in column.docs.lines() {
                        writeln!(out, "///{}{}", if d.is_empty() { "" } else { " " }, d)?;
                    }
                }
                if column.rust_name == column.sql_name {
                    writeln!(out, "{} -> {},", column.sql_name, column.ty)?;
                } else {
                    writeln!(out, r#"#[sql_name = "{}"]"#, column.sql_name)?;
                    writeln!(out, "{} -> {},", column.rust_name, column.ty)?;
                }
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

struct Joinable<'a>(&'a ForeignKeyConstraint);

impl<'a> Display for Joinable<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let child_table_name = &self.0.child_table.rust_name;

        let parent_table_name = &self.0.parent_table.rust_name;

        write!(
            f,
            "diesel::joinable!({} -> {} ({}));",
            child_table_name, parent_table_name, self.0.foreign_key_rust_name,
        )
    }
}

/// Lifted directly from libcore/fmt/builders.rs
struct PadAdapter<'a, 'b: 'a> {
    fmt: &'a mut Formatter<'b>,
    on_newline: bool,
}

impl<'a, 'b: 'a> PadAdapter<'a, 'b> {
    fn new(fmt: &'a mut Formatter<'b>) -> PadAdapter<'a, 'b> {
        PadAdapter {
            fmt,
            on_newline: false,
        }
    }
}

impl<'a, 'b: 'a> Write for PadAdapter<'a, 'b> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            let on_newline = self.on_newline;

            let split = match s.find('\n') {
                Some(pos) => {
                    self.on_newline = true;
                    pos + 1
                }
                None => {
                    self.on_newline = false;
                    s.len()
                }
            };

            let to_write = &s[..split];
            if on_newline && to_write != "\n" {
                self.fmt.write_str("    ")?;
            }
            self.fmt.write_str(to_write)?;

            s = &s[split..];
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for Filtering {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FilteringVisitor;

        impl<'de> Visitor<'de> for FilteringVisitor {
            type Value = Filtering;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("either only_tables or except_tables")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut only_tables = None::<Vec<Regex>>;
                let mut except_tables = None::<Vec<Regex>>;
                while let Some(key) = map.next_key()? {
                    match key {
                        "only_tables" => {
                            if only_tables.is_some() {
                                return Err(de::Error::duplicate_field("only_tables"));
                            }
                            only_tables = Some(map.next_value()?);
                        }
                        "except_tables" => {
                            if except_tables.is_some() {
                                return Err(de::Error::duplicate_field("except_tables"));
                            }
                            except_tables = Some(map.next_value()?);
                        }
                        _ => {
                            return Err(de::Error::unknown_field(
                                key,
                                &["only_tables", "except_tables"],
                            ))
                        }
                    }
                }
                match (only_tables, except_tables) {
                    (Some(t), None) => Ok(Filtering::OnlyTables(t)),
                    (None, Some(t)) => Ok(Filtering::ExceptTables(t)),
                    (None, None) => Ok(Filtering::None),
                    _ => Err(de::Error::duplicate_field("only_tables except_tables")),
                }
            }
        }

        deserializer.deserialize_map(FilteringVisitor)
    }
}
