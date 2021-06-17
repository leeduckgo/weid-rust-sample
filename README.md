# weid-rust-example

使用 Rust 和 WeIdentity/WeIdentity-Rest-Service交互的案例。

WeIdentity文档：

> https://weidentity.readthedocs.io/zh_CN/latest/index.html

WeIdentity-Rest-Service 文档：

> https://weidentity.readthedocs.io/zh_CN/latest/docs/weidentity-rest-api.html

## 运行项目

1. 初始化数据库

```bash
./bin/diesel database reset
```

2. 设置环境变量

```bash
# 推荐使用direnv
export DATABASE_URL="examples.db"
export BACKEND="sqlite"
export WEID_URL=<weid-rest-service url>
```

3. 运行项目

```
RUST_LOG=info cargo run
```

目前会在链上创建托管型`WeId`并存储在本地的`Sqlite`数据库中。

![image-20210617172840467](https://tva1.sinaimg.cn/large/008i3skNgy1grle649q4zj30ll0a2dhd.jpg)

## 系列教程

本案例配套系列教程：

https://mp.weixin.qq.com/mp/homepage?__biz=MzI0NTM0MzE5Mw==&hid=9&sn=44aa3f3183dbddd7844f8507d9dc6aab

