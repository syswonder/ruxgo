# 运行不同的app

在`ruxgo/apps/`目录下放置了所有经过测试的 Toml 文件。目前，有两种方法构建应用程序：

- 如果在本地构建，你只需要在`ruxgo/apps/<name>/local`目录下下载 app 的源代码，然后使用 ruxgo 构建并运行它。

- 如果在 RuxOS 上构建，你需要将`config_linux.toml`从`ruxgo/apps/<name>/ruxos`复制到`ruxos/apps/c/<name>`，然后下载 app 的源代码并使用 ruxgo 来构建并运行它。

**注:** 有关详细信息，请参阅每个 app 目录下的 README.md。以下应用程序已获支持:

* [x] [redis](https://github.com/syswonder/ruxgo/tree/master/apps/redis)
* [x] [sqlite3](https://github.com/syswonder/ruxgo/tree/master/apps/sqlite3)
* [x] [iperf](https://github.com/syswonder/ruxgo/tree/master/apps/iperf)
* [x] helloworld
* [x] memtest
* [x] httpclient
* [x] httpserver
* [x] nginx
