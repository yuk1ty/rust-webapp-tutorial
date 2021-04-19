# second-todo-list

このチュートリアルでは、アプリケーションをデータベースに接続する作業を行います。タスクをデータベースに登録したり、あるいはデータベースに登録されているタスクの一覧を JSON で返すといったエンドポイントを作ります。

## 事前準備

データベースには SQLite を使用します。なので、手元の環境に SQLite がない方は、まずそちらをご用意ください。

下記のサイトよりダウンロードするか、あるいはお手元の OS のパッケージマネージャを使用してインストールしてください。

- https://www.sqlite.org/download.html

## データベースに接続するクレートを追加する

今回は下記のクレートを使用して、SQLite に接続してみます。

Cargo.toml に下記依存を追加してください。

```toml
r2d2 = "0.8.9"
r2d2_sqlite = "0.18.0"
rusqlite = { version = "0.25.0", features = ["chrono", "uuid"] }"
```

- `r2d2`: コネクションプールを管理する際に使用するクレートです。
- `r2d2_sqlite`: r2d2 と rusqlite を統合して使用できるようにするクレートです。
- `rusqlite`: SQLite 向けにクエリの実行などを行えるようにするクレートです。

rusqlite が features に `chrono` と `uuid` をもつのは、内部で UUID や日付型を自動で SQL 用に変換してくれると嬉しいからです。この features をオンにするとそれが可能になります。

## 新しいデータ構造を用意する

今回作るのは JSON をボディにもつ HTTP リクエストを POST で送ると、JSON の内容をパースしてデータベースにタスク内容を保存するエンドポイントを作成します。

前回のチュートリアルで作成した `Todo` という構造体がそのまま使えるのではないか？と思うかもしれませんが、下記の理由から `Todo` 構造体はそのままでは使用できません。

- JSON リクエストの内容的に、`Todo` 構造体をそのまま使うと無駄が多い。

なので、HTTP リクエストに関しては特別に構造体を用意します。

今回は下記のような JSON リクエストを送ろうとしています。

```json
{
  "description": "Rustハンズオン準備"
}
```

`Todo` 構造体で用意したような `done` や `datetime` などはリクエストに含まれません。不要なフィールドが多いので、便宜的に別の構造体を作って対応することにします。

```rust
#[derive(Deserialize)]
struct RegisterTodo {
    description: String,
}
```

serde の `Deserializable` を有効にし、先ほどの JSON フィールドに対応したフィールドを構造体に用意します。これで、一旦 JSON リクエストを受け取るための準備は完了です。

## リクエストを受け取って RDS に保存するエンドポイントを作る

HTTP リクエストを受け取って RDS に保存するエンドポイントを作っていきます。2 ステップあります。

1. main 関数に SQLite への接続を追加し、データベース接続できることを確かめる。
2. 新しく `register_todo` というエンドポイントを作成する。

### 1. main 関数に SQLite への接続を追加し、データベース接続できることを確かめる。

コネクションプールを作って、RDS への接続をできるようにします。下記コードをまずは見てみましょう。

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    let manager = SqliteConnectionManager::file("test.db");
    let pool = Pool::new(manager).unwrap();

    info!("Bootstrapping the server...");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(hc)
            .service(todo_list)
            .service(register_todo)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

前回の first-todo-list のケースと比べると、`manager` と `pool` という変数がまず増えていることがわかります。そして、`App` では `data` というメソッドを用いて、`pool` が引数に入れられていることがわかります。ほかにもいくつか差異はありますが、それは後ほど解説するので、まずはこの 2 つの差分を確認していきましょう。

#### `manager` と `pool`

SQLite が管理するファイルからコネクションプールを生成します。

```rust
    let manager = SqliteConnectionManager::file("test.db");
    let pool = Pool::new(manager).unwrap();
```

### `App#data`

`data` メソッドを使ってアプリケーション全体で使用するデータを保存することができます。これはグローバルな状態を共有する手段です。

```rust
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            // ...
    })
```

`data` で登録したものは、後ほど使い方を確認しますが `Data<T>` という型によって取り出し可能です。登録したコネクションプールは、後ほど登録用のエンドポイントにて取り出されて使用されます。
