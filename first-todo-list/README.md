# first-todo-list

このチュートリアルでは、JSON を使用した HTTP レスポンスを返すためのエンドポイントを作ってみます。具体的には `/todo` というエンドポイントに対して GET リクエストを送ると、HTTP ステータス 200 で、レスポンスボディに設定したタスクの一覧が JSON で返されるようなエンドポイントを作ります。

## 事前準備

Cargo.toml あるいは cargo-edit を用いて、下記クレートの依存を追加してください。`feature` に serde がついているものは、後ほど使用するものですのでそれも忘れないように追加してください。

```toml
[dependencies]
actix-web = "3.3.2"
serde = { version = "1.0.125", features = ["derive"] }
chrono = { version = "0.4.19", features = ["serde"] }
uuid = { version = "0.8.2", features = ["serde", "v4"] }
log = "0.4.14"
env_logger = "0.8.3"
```

## Rust で JSON ⇔ 構造体の変換を扱うためには

serde という Rust のデファクトのような立ち位置のクレートを利用します。serde は JSON だけではなく、他のざまざまなデータ構造との変換を行うことができるクレートです。

### 簡単な JSON 変換

今回はタスクを表現する Todo という構造体を作ってみます。この Todo は、HTTP レスポンスボディに JSON の形式で入れられ、レスポンス時に使用されるものとします。

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid

#[derive(Serialize)]
struct Todo {
    id: Uuid,
    description: String,
    done: bool,
    datetime: DateTime<Utc>,
}
```

まず、構造体から JSON に変換可能にするためには `Serialize` というトレイトを継承させます。これにより自動で構造体から JSON への変換を裏でかけてくれるようになるため、使う側は JSON の変換処理について意識することはほとんどありません。

`Serialize` できるためには、構造体がもつフィールドの型も `Serialize` を継承している必要があります。なのでたとえば、`Uuid` や `DateTime<Utc>` といった型も `Serialize` トレイトを継承している必要があります。今回はサードパーティのクレート `uuid` と `chrono` を使用しつつ、`feature` で `serde` への変換を実装済みの状態にしてありますので、とくに追加で実装が必要なことはありません。

2 つほど新しいクレートが出てきたので、さらに解説を加えます。

### uuid クレート

UUID を生成できます。今回は UUIDv4 を生成して ID を採番したいので、このクレートを使用することにしています。

### chrono クレート

`DateTime` 型は chorono というクレートが提供する型です。日時操作に関する Rust におけるデファクトスタンダードになっています。

### フィーチャーフラグ

どちらも、`Cargo.toml` にてフィーチャーフラグ `features = ["serde"]` をオンにしています。これにより、`Serialize` が継承された状態で使用できるので、使う側は特別に `derive(Serialize)` を追加で行う必要はありません。

フィーチャーフラグというのは、コンパイル時に特定の機能をオン／オフ切り替える際に使用できる機能です。クレートによっては今回の serde 機能オンのように、いくつか `features` が用意されていて、その中から必要なもののみ選択して使用するといった使い方ができるものがあります。フィーチャーフラグを導入しておくと、不要な機能のコード箇所はコンパイルしないので、その分コンパイル速度の向上が見込めるなどのメリットがあります。

### Todo のリストを表現する構造体を用意する

同様に Todo のリストを表現する構造体を用意しましょう。

```rust
#[derive(Serialize)]
struct TodoList(Vec<Todo>);
```

こうして下記のように `Todo` オブジェクトのリストを内部にもつ JSON を返すことができるようになります。下記は例です。

```rust
[
    {
        "id": "873192b0-5c5a-4cce-8b5d-299ddde5062e",
        "description": "タスク1",
        "done": false,
        "datetime": "2021-04-14T09:26:39.346153Z"
    },
    {
        "id": "903b9443-0802-45e2-906d-64d9317e7ef5",
        "description": "タスク2",
        "done": false,
        "datetime": "2021-04-14T09:26:39.346160Z"
    }
]
```

ここでのポイントは NewType パターンと呼ばれるものです。 `struct TodoList(Vec<Todo>)` という書き方です。今回 NewType パターンを導入したのは、単純にこうすると `Todo` のリストのシリアライズを簡単にできるからという理由ではあるのですが、NewTYpe パターン自体は Rust で多くみかけるので、解説します。

### New Type パターン

New Type の出自を正確には知らないのですが、私は Haskell を書いているときにこの用語を初めて見たように記憶しています。`newtype` は Haskell においては、ある型を元に別の新たな型を作る記法として導入されています。

Rust では明示的に `newtype` のようなキーワードがあるわけではありませんが、New Type を Haskell 同様に実現できます。`struct A(T)` のように、型情報を構造体にもたせることで実現できます。実質要素 1 つのタプルをもたせたことになるので、取り出し時は `.0` のように通常のタプルと同じ記法で取り出しができます。

Rust の公式の解説書である[ The Rust Programming Language ](https://doc.rust-jp.rs/book-ja/ch19-04-advanced-types.html)という本にも NewType パターンに関して言及している箇所があり、広く普及しているイディオムだと筆者は思います。

New Type パターンは、型に情報を付与して表現力を強めたり、あるいは型内部の実装情報を隠蔽するために使用したりします。[実例はこの記事が詳しいです](https://keens.github.io/blog/2018/12/15/rustdetsuyomenikatawotsukerupart_1__new_type_pattern/)。

## ダミーで作ったタスクのリストを返すエンドポイントを用意する

GET リクエストを送ると、ダミーで用意したタスクのリストを返すようなエンドポイントを用意してみましょう。`/todo` というエンドポイントを用意し、それを HTTP サーバーに登録します。

```rust
#[get("/todo")]
async fn todo_list() -> impl Responder {
    let list = TodoList(vec![
        Todo {
            id: Uuid::new_v4(),
            description: "タスク1".to_string(),
            done: false,
            datetime: Utc::now(),
        },
        Todo {
            id: Uuid::new_v4(),
            description: "タスク2".to_string(),
            done: false,
            datetime: Utc::now(),
        },
    ]);
    HttpResponse::Ok().json(list)
}
```

作成したこの関数を、`App` に登録します。

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hc).service(todo_list))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
```

## サーバーでログを出したい

サーバーを起動した際にログを出力したいと思うかもしれません。この節では Rust でログを出すためにはどうすればよいかについて、使用するといいクレートや Rust のログの考え方について紹介します。
