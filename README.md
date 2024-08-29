## 简易版 Redis 服务器

redis serialization (RESP) protocol specification

[Redis serialization protocol specification | Docs](https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-strings)

如何解析 客户端的命令。

创建一个新项目，simple-redis

构建`RespFrame`这样一个数据结构体,完成以下数据类型的解析（符合 RESP）

```shell
Simple strings
	- +OK\r\n
Simple errors
	- -Error message\r\n
Bulk errors
	- !<length>\r\n<error>\r\n
Integers
	- :[<+|->]<value>\r\n
Bulk strings
	- $<length>\r\n<data>\r\n
Null bulk strings
	- $-1\r\n
Arrays
	- *<number-of-elements>\r\n<element-1>...<element-n>
Null arrays
	- *-1\r\n
Nulls
	- _\r\n
Booleans
	- #<t|f>\r\n
Doubles
	- ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
Maps
	- %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
Sets
	- ~<number-of-elements>\r\n<element-1>...<element-n>
```

针对以上列举出来的数据类型，构建一个类型的`enum`
