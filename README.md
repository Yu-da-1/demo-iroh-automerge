# Automerge
irohとautomergeの[example](https://github.com/n0-computer/iroh-examples/tree/main/iroh-automerge)をフォークした。
試したいことはCRDTの実装である。
- [iroh](https://www.iroh.computer/)
- [automerge](https://automerge.org/)

## How to run?

```shell
# 1つ目のターミナル
cargo run
Running
Node Id: rgkuma5fkzzywshcw24wabtvhvywhlwzsiu6f3ha7ubdtytfkdfa
```
1つ目のターミナルで表示された`node id`を2つ目のターミナルで`remote-id`以降に入れる。
```shell
# 2つ目のターミナル
> cargo run -- --remote-id rgkuma5fkzzywshcw24wabtvhvywhlwzsiu6f3ha7ubdtytfkdfa
Running
Node Id: lkpz2uw6jf7qahl7oo6qc46qad5ysszhtdzqyotkb3pwtd7sv3va
Enter a key and value separated by a space (or 'exit' to quit): 
```
Enter以降にkeyとvalueを入力すると、1つ目に立てたノードに送信される。

例えば、key: name , value: John
```
User input received: name John
Sending data: key = name, value = John
Data committed and merged.
Retry count: 0
Connected successfully.
Sync completed successfully
```

1つ目のノードには以下が表示される。
```
Node Id: rgkuma5fkzzywshcw24wabtvhvywhlwzsiu6f3ha7ubdtytfkdfa
Document received. Processing state...
name => "John"
Document processed.
```

ノードの終了
```ssh
exit
```
