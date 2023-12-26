# Build and Run [`redis`](https://github.com/redis/redis) in two ways and benchmark

## 1. Build and run locally

Firstly, clone github repository of redis and configure it: 

```bash
wget https://github.com/redis/redis/archive/7.0.12.tar.gz
tar -zxvf 7.0.12.tar.gz && rm -f 7.0.12.tar.gz
cd redis-7.0.12/src && ./mkreleasehdr.sh && cd ../..
```

Then, you need to copy `config_linux.toml` from `ruxgo/apps/redis/local` and place it in the `redis-7.0.12/` directory you just downloaded. 

Finally, cd into `redis-7.0.12`/ and execute the following commands to build and run it:

```bash
# Build and Run
ruxgo -b
ruxgo -r
```

**Note:** 

`ruxgo -r` performs the default configuration `redis.conf`, which is in the `redis-7.0.12/`, you can modify it. However, If you want to provide your `redis.conf`, you have to run it using an additional parameter (the path of the configuration file):

```
ruxgo -r --bin-args=/your_path/redis.conf
```

It is possible to alter the Redis configuration by passing parameters directly as options using the command line. Examples:

```
ruxgo -r --bin-args=--port,9999,--loglevel,debug
```

All the options in `redis.conf` are also supported as options using the command line, with exactly the same name.

## 2. Build and run on RuxOS:

Firstly, you need to copy `config_linux.toml` from `ruxgo/apps/redis/ruxos` and place it in the `ruxos/apps/c/redis` at the same level as `redis-7.0.12`.

Then, switch to `ruxos/apps/c/redis` directory. If `redis-7.0.12` does not exist in the `ruxos/apps/c/redis` directory, execute the following prerequisite commands (if it does, it is not required):

```bash
wget https://github.com/redis/redis/archive/7.0.12.tar.gz
tar -zxvf 7.0.12.tar.gz && rm -f 7.0.12.tar.gz
cd redis-7.0.12/src && ./mkreleasehdr.sh && cd ../..
```

Finally, execute the following commands to build and run it:

```bash
# Build and Run
ruxgo -b
ruxgo -r
```

**Note:** 

If 9pfs is not used, the args field in the `config_linux.toml` file is: 

```
args="./redis-server,--bind,0.0.0.0,--port,5555,--save,\"\",--appendonly,no,--protected-mode,no,--ignore-warnings,ARM64-COW-BUG"
```

If 9pfs is used, the args field in the `config_linux.toml` file is: 

```
args="./redis-server,/v9fs/redis.conf"
```

You can choose your own `redis.conf` or copy `redis.conf` from `ruxgo/apps/redis/ruxos` and place it in the `ruxos/apps/c/redis` .

## 3. Benchmark

- Use `redis-cli -p 5555` to connect to redis-server, and enjoy Ruxos-Redis world!
- Use `redis-benchmark -p 5555` and other optional parameters to run the benchmark.
  - Like: `redis-benchmark -p 5555 -n 5 -q -c 10`, this command issues 5 requests for each commands (like `set`, `get`, etc.), with 10 concurrency.