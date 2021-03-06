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

さっそくですが、HTTP サーバーの立ち上げをまずは行ってみましょう。今回は、まずは `localhost:8080` に対してサーバーを立ち上げてみます。

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new())
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
```

ひとつひとつ要素を解説していきます。

### `HttpServer` と `App`

`HttpServer` 構造体は、HTTP サーバーそのものの設定や起動などを司る構造体になっています。今回は `bind` と `run` しか使用していませんが、コネクションの数の設定 (`max_connections`) や Keep-Alive の設定 (`keep_alive`) といったサーバー自体の細かい設定も、この構造体によって行うことができます。

`App` 構造体は HTTP サーバーの上で動かされるアプリケーション側の設定を行います。具体的には、これから実装することになりますがエンドポイントの設定を行ったり、データベースへのコネクションプールをアプリケーション全体で保持させたり、あるいはアプリケーション全体でなにか共有したい状態などがあればそういったものも保持させることができます。

両者ともにメソッドチェーンの連なる、いわゆるビルダーパターンが使用されていることに気づいた方もいらっしゃるかもしれません。Rust には名前付き引数がないので、このようにビルダーパターンで設定を追加して欲しい構造体をビルドするといった手法がよく利用されます。

### `async`/`.await` と `#[actix_web::main]`

HTTP サーバーを構築する上で必須になってくるのが、非同期処理あるいはノンブロッキング I/O 多重化と呼ばれる概念です。この記事では深くは解説しませんが、たとえば[ nginx が対応するような課題である C10K 問題の解決の際にはキーワードとして登場してくることが多い](https://www.nginx.co.jp/blog/what-is-nginx/)です。`async`/`await` はこうした問題に立ち向かうための解決策を提供してくれます。

`async` や `await` というキーワードは、JavaScript や C# などで見かけると思いますが、使い心地はそうした言語とほぼ変わりありません。`async` で非同期化したい処理を定義しておき、`await` で処理をそこまで進める、ということを繰り返し行うというのが基本コンセプトです。

Rust の場合は、`async` はある概念のシンタックスシュガーになっています。それは `Future` です。Rust には `Future` がありますが、これを素で扱おうとすると、コールバックが増えたり型情報が複雑になったりといったデメリットがありました。プログラマがそれらを意識しながら書くのが大変手間だったのと、その他いくつか解決する必要のある問題が存在したことから、`async` / `.await` というキーワードが導入されることになりました。

Rust の `async`/`await` は非常に奥が深いです。もし深く理解したい場合には、次の記事を一読されることをおすすめします。

- Rust の非同期プログラミングをマスターする: https://tech-blog.optim.co.jp/entry/2019/11/08/163000
- Rust でお気楽非同期プログラミング: https://qiita.com/Kumassy/items/fec47952d70b5073b1b7

`#[actix_web::main]` というのはアトリビュートであり、処理の裏側は実はマクロが呼び出されています。これは手続きマクロと呼ばれるもので、Rust のプログラム自体を解析しながら共通する処理をするコードを自動で生成できるなどの機能を持ちます。今回の利用例でも、ご多分に漏れずこのマクロはコードを生成します。

このマクロがどのようなコードを生成しているかは、[`cargo-expand`](https://github.com/dtolnay/cargo-expand) というツールを見ると確認することができます。下記は展開してみた結果です。

```rust
// 展開前のコード↓
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| App::new().service(hc))
//         .bind("127.0.0.1:8080")?
//         .run()
//         .await
// }

fn main() -> std::io::Result<()> {
    actix_web::rt::System::new("main").block_on(async move {
        {
            HttpServer::new(|| App::new())
                .bind("127.0.0.1:8080")?
                .run()
                .await
        }
    })
}
```

先ほどあった `#[actix_web::main]` が消え、先ほどはなかったコードが挿入されていることがわかります。`actix_web::rt::System` はアクターシステムの起動をしており、`block_on` は `async` で定義された一連の処理情報を待ち受け実行する役割をもつ関数です。

Rust の非同期処理は、`async` / `.await` によって生成される一連の `Future` のチェーンと、それら非同期処理を実行する基盤の組み合わせで成立しています。非同期処理基盤にはいくつか種類があり、

- tokio: https://github.com/tokio-rs/tokio
- async-std: https://github.com/async-rs/async-std

といったクレートをよく見かけることになるはずです。

tokio と async-std は目指す先が少し違います。async-std は、Rust が提供する標準 API をすべて非同期化することを目指して作られているランタイムです。tokio は独自の API もちます。

両者の性能差が話題になっていることは見たことがなく、[ベンチマークをとった方の記事を読んでみても大差ないように思われます](https://medium.com/nttlabs/rust-async-runtime-comparison-7a79a4477fed)。なので現時点では、本当に好みの問題で選ぶということになってしまいそうです。

ではどれを選ぶか、という話になると思うのですが、エコシステムの充実度合いなどから現時点では tokio を推薦しておきます。tokio に対応したクレートが圧倒的に多く、「これを使いたい！」と思ったときに対応するクレートがすぐ見つかるのも tokio のほうが経験上は多いように思います。

非同期処理基盤が複数あることに違和感を覚える方もいるかもしれません。が、Rust は組み込み開発から Web までかなり幅広い領域で利用される言語です。そうした幅広い領域すべてをカバーできる非同期処理基盤を作るのは至難の業です。ある程度目的に応じて基盤が存在したほうが、ユーザーにとっても嬉しいはずです。そうした目的別の非同期処理基盤があり、ユーザーはそこから最適なものを選ぶことができるようになっています。

## ヘルスチェックエンドポイントの定義

次にヘルスチェックの結果を返すエンドポイントを定義してみます。ここにも見慣れない文法がいくつかあると思います。

```rust
use actix_web::{get, Responder};

#[get("/health")]
async fn hc() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
```

### `#[get("/health")]`

`#[get("/health")]` はやはりマクロになっています。このマクロも展開してみると下記のような処理が裏側で行われているとわかります。actix-web が保有する `HttpServiceFactory` というトレイトを実装し、後ほど `App` にエンドポイントを登録する際にこの `register` 関数が呼び出されるという構成になっています。その他にもいろいろなコードが展開されていますが、ここでは深くは立ち入らないことにします。

```rust
#[allow(non_camel_case_types, missing_docs)]
pub struct hc;
impl actix_web::dev::HttpServiceFactory for hc {
    fn register(self, __config: &mut actix_web::dev::AppService) {
        async fn hc() -> impl Responder {
            HttpResponse::Ok().body("OK")
        }
        let __resource = actix_web::Resource::new("/health")
            .name("hc")
            .guard(actix_web::guard::Get())
            .to(hc);
        actix_web::dev::HttpServiceFactory::register(__resource, __config)
    }
}
```

### `impl Responder`

`impl Responder` というのは Rust の[ `impl Trait` という機能](https://doc.rust-lang.org/rust-by-example/trait/impl_trait.html)を利用しています。impl Trait は戻り値型を隠蔽する機能を提供します。`impl Responder` の場合、`Responder` トレイトを実装した型であれば暗黙的に裏で型を一致させて返してくれるようになります。

impl Trait を使用すると型を簡潔に書けるというメリットがあります。この Responder の型も、`impl Responder` を使用しない場合は `Responder<Future = ..., Error = ...>` のような型になってしまい、かなり煩雑な見た目になります。それを防ぐことができます。

### レスポンスを返す

レスポンスを返しているのはこの箇所です。`body` を使うとレスポンスボディに特定の情報を詰めて返すなどといったことができます。

```rust
HttpResponse::Ok().body("OK")
```

今回は `&str` 型を入れていますが、他には `&[u8]` などの型が入ります。`body` の引数の型は `Into<Body>` を実装した型になっていれば何でもよく、自前で型を定義して、それに `Into` を実装して引数に入れることも可能です。

ステータスコードはもちろんですが `Ok` だけでなく、`InternalServerError` などを入れることもできます。

```rust
HttpResponse::InternalServerError()
```

## エンドポイントをサーバーに登録する

`App` に作ったエンドポイントを最後に登録してみましょう。`App` の `service` というメソッドに先ほど作った `hc` 関数を入れると登録をすることができます。最終的にできあがるコードを載せます。

```rust
use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/health")]
async fn hc() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hc))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
```
