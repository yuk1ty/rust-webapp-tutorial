This document is written in Japanese, but I'm planning to append the English version! Just a moment!

# health-check

このチュートリアルでは、ヘルスチェックを行えるエンドポイントを作ってみます。具体的には `/health` というエンドポイントに対して GET リクエストを送ると、HTTP ステータス 200 でレスポンスボディに `"OK"` という文字列を入れて返すエンドポイントをまず手始めに作ります。

## 事前準備

Cargo.toml に actix-web への依存を追加しましょう。いくつか方法がありますが、

- Cargo.toml に直接書く場合
- cargo add actix-web で追加する場合

があります。

### Cargo.toml に直接書く場合

下記を `[dependencies]` に追加してください。

```
actix-web = "3.3.2"
```

### cargo add actix-web で追加する場合

`cargo-edit` というツールを使用してクレートを追加することもできます。`cargo install cargo-edit` で追加可能です。

https://github.com/killercup/cargo-edit

バージョンを指定してクレートを追加したい場合は、`@x.y.z` というようにアットマークを追記するとできます。

```
cargo add actix-web@3.3.2
```

クレートの追加が終わったら、次に行きましょう。

## actix-web について

actix-web は、Actix という独自のアクターシステムを基盤とした HTTP サーバーフレームワークです。actix-web は [Actix](https://github.com/actix/actix) の上に Web を扱いやすくするためにいくつかラッピングがされているので、actix-web を使用しているうちはほとんど Actix が管理するアクターを触る機会はありません。

Actix は tokio のランタイムの上に載っているので、実質 tokio ベースと言えます。中ではさらに [actix-rt](https://github.com/actix/actix-net/tree/master/actix-rt) というクレートを呼び出しており、この実装を見ると tokio をいろいろ利用しながら独自のランタイムを作っていることがわかります。

## HTTP サーバーの立ち上げとエンドポイントの追加

さっそくですが、HTTP サーバーの立ち上げをまずは行ってみましょう。

actix-web では `HttpServer` という構造体が HTTP サーバーそのものを表現しています。
