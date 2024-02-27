##  Understanding Performance Characteristics Of LSM Trees

### Quick Overview Of LSM-tree functionality
They have an in-buffer append-only memtable. When the memtable reaches a certain size it is frozen to an immutable memtable and then written to disk as an SSTable at some later point.

Most implementations of LSM-trees use SkipLists(?) so insertion should be O(log(N)) and sorted order is maintained.

SSTables are compacted later with different strategies. Strategies are laid out [here](https://github.com/facebook/rocksdb/wiki/Compaction). The most common strategy seems to be leveled compaction (if only because it seems to be the oldest).


### Handling writes
When doing writes to an LSM engine, writes are made to a WAL first and then the memtable ([optional on RocksDB](https://github.com/facebook/rocksdb/wiki/RocksDB-Overview#3-high-level-architecture)). 

There's a trade-off to make here in terms of throughput vs durability:
1. Flush the WAL (fsync) on every write (or transaction commit). This guarantees durability at the cost of write throughput.
2. Checkpoint the WAL based on certain metrics. This improves write throughput but you lose durability if the system crashes before checkpoint.

*Note: I believe that instead of committing on every transaction commit, you could potentially do a steal + no-force policy to not have to wait for a checkpoint to commit but still write to disk at regular intervals. But, I'm not going into that because I don't understand it completely.*

When the memtable is frozen, the WAL can be truncated or deleted and a fresh WAL started. When the memtable is frozen, it's typically done via a dedicated thread pool to avoid stalled writes ([Rocks DB reference](https://github.com/facebook/rocksdb/wiki/RocksDB-Overview#avoiding-stalls)). 

Truncating the log should only be done once the background thread freezing the memtable has joined. In case the thread crashes, the log can be used to re-create the original memtable and freeze it again.

### Handling reads
When doing reads, range scans or key lookups use an iterator that travels in reverse chronological direction (newest to oldest) to find the key. Depending on the levels of compaction, this could potentially involve reading a lot of files from disk.

A typical optimisation here is bloom filter, one for each SST, which allows you to filter out certain SST's. Using a block cache also seems like a typical optimisation.

### Discussing Amplification (comparison with B-trees)
This was one of the more confusing topics. I'm going to preface this by saying that the answer to most questions about amplification seems to come down to "it depends".

Getting some definitions out of the way (taken from [Mark Callaghan's blog](https://smalldatum.blogspot.com/2015/11/read-write-space-amplification-pick-2_23.html)):
1. Read Amplification -> amount of work done per logical read operation
2. Write Amplification -> amount of work done per write operation
3. Size Amplification -> ratio of the size of the database to the size of the data in the database

[Note: The [TiKV blog](https://tikv.org/deep-dive/key-value-engine/b-tree-vs-lsm/) seems to use different definitions. But I find those confusing].

####  Read Amplification
Here, it seems straightforward that LSM trees have higher read amplification than B-trees on average. Assuming block size B, data size N and number of children per node C and assuming a cold start with empty cache, a B-tree would require O(log(N/B)) to the base C number of disk seeks.

I'm not sure how to quantify the read amplification of the LSM trees. But, intuitiely it makes sense that there is greater read amplification because every update is an append rather than an in-place update like a B-tree.

####  Space Amplification
Initially, I assumed the answer here would be that LSM-trees have greater space amplification because each update is an append. However, I'm not a 100% sure that's necessarily true. I think it would really depend on the workload.

Page fragmentation in B+ trees can also cause space amplification. Good compression in LSM trees can reduce space amplification and this would depend on the nature of data being written. Also, see [Mark Callaghan's reply here](https://x.com/MarkCallaghanDB/status/1761212486798528612?s=20) to me on Twitter. It seems to depend on the workload.

####  Write Amplification
This was the trickiest to figure out and the best I could come up with is "it depends".

My initial assumption was that the write amplification in LSM-trees is greater because the append occurs(in-memory buffer) and then freezing and compaction. Freezing and compaction both involve disk writes. For a B-tree there's a single update and that's it.

But, both [Mark Callaghan's blog](https://smalldatum.blogspot.com/2015/11/read-write-space-amplification-pick-2_23.html) and the [TiKV blog](https://tikv.org/deep-dive/key-value-engine/b-tree-vs-lsm/) both say write amplification in B-trees is higher. In a world with no WAL and buffer pool, this would make sense but no B-tree does a write to disk every time. It batches writes via buffer pool.

When posting this on Twitter, [Sunny Bains' reply](https://x.com/sunbains/status/1760730549264732361?s=20) seems to suggest that it depends on the WAL and cache. Without either, a B-tree update would result in a full page disk write. However, with the two it doesn't.

Honest answer: I'm unsure of this.

### Hardware
The two classic choices are -> SSD and HDD. SSD is able to do random reads and writes to any cell instantaneously while HDD's have mechanical moving parts which take longer and incur wear and tear.

SSD cells have a [limited number of writes per cell](https://superuser.com/questions/1107320/why-do-ssd-sectors-have-limited-write-endurance) and the cost per GB of storage is higher.

Given the write optimised nature of LSM-trees and the need to read from multiple files ([I'm unsure if HDD's can perform parallel I/O](https://pkolaczk.github.io/disk-parallelism/), probably not?) to perform compaction, SSD's would be a better option. Another reason could be that after repeated compactions there is a chance the SST files could be fragmented and the sequential read nature of SST's could devolve into more random I/O which would benefit more from SSD's.

### Additional Note

[I like Justin Jaffrey's idea](https://buttondown.email/jaffray/archive/the-three-places-for-data-in-an-lsm/) of breaking down the LSM into its component parts and analysing the performance of each.