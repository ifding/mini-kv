
# A mini kv database demo that using simplified Bitcask 

Key-value stores are similar to a hash map (hash table) in most programming languages. Let's say our data storage consists only of appending to a file. Then the simplest indexing strategy is: keep an in-memory hash map where every key is mapped to a byte offset in the data file - the location at which the value can be found, as illustrated:

![](resources/fig3-1.png)

Whenever you append a new key-value pair to the file, you also update the hash map to reflect the offset of the data you just wrote (this works both for inserting new keys and for updating existing keys). When you want to look up a value, use the hash map to find the offset in the data file, seek to that location, and read the value. In fact, this is essentially what `Bitcask` does. The values can be loaded from disk with just one disk seek. 

A storage engine like `Bitcask` is well suited to situations where the value for each key is updated frequently. For example, the key might be the URL of a cat video, and the value might be the number of times it has been played (incremented every time someone hits the play button). In this kind of workload, there are a lot of writes, but there are not too many distinct keys - you have a large number of writes per key, but it's feasible to keep all keys in memory. 
(From the book `Designing Data-Intensive Applications` by Martin Kleppmann)

From the above key-value stores, there are two main tasks we need to solve:

- The In-memory hash map contains `key` and `byte offset`, how to write/read the key-value pairs into the log-structured file on the disk. We need to encode/decode the data.
- How do we avoid eventually running out of disk space? We need to perform _compaction_ to throw away duplicate keys in the log, and keeping only the most recent update for each key.


## Encode/Decode data


## Reference

- <https://github.com/Morgan279/miniDB>