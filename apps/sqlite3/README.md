# Build and Run [`sqlite3`](https://github.com/sqlite/sqlite) in two ways 

## 1. Build and run locally

Firstly, get the sqlite3 source code: 

```bash
wget https://sqlite.org/2023/sqlite-amalgamation-3410100.zip
unzip sqlite-amalgamation-3410100.zip && rm -f sqlite-amalgamation-3410100.zip
```
Then, you need to copy `config_linux.toml` and `main.c` from `ruxgo/apps/redis/local` into the same directory as `sqlite-amalgamation-3410100` that you just downloaded.

Finally, execute the following commands to build and run it:

```bash
# Build and Run
ruxgo -b
ruxgo -r
```

## 2. Build and run on RuxOS:

Firstly, you need to copy `config_linux.toml` from `ruxgo/apps/sqlite3/ruxos` and place it in the `ruxos/apps/c/sqlite3` at the same level as `sqlite-amalgamation-3410100`.

Then, switch to `ruxos/apps/c/sqlite3` directory. If `sqlite-amalgamation-3410100` does not exist in the `ruxos/apps/c/sqlite3` directory, execute the following prerequisite commands (if it does, it is not required):

```bash
wget https://sqlite.org/2023/sqlite-amalgamation-3410100.zip
unzip sqlite-amalgamation-3410100.zip && rm -f sqlite-amalgamation-3410100.zip
```

Finally, execute the following commands to build and run it:

```bash
# Build and Run
ruxgo -b
ruxgo -r
```