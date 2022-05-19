# telegram-mesaga-fonto
## 目前能做的事情
让指定ID用户可以设置/解绑频道，这么干的：
```rust
if (admin.user.id == sender_id || sender_id.to_string() == "114514")
```
## TODO
* 屏蔽某个人
* 防止bot被拉进奇奇怪怪的群
## ~~不太可能DO的TODO~~
* 使用配置文件而不是直接改源码（逃
