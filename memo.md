# 理想
__**## プリフロップレンジについて、いくつかのサンプルから選択できる。**__
* 標準
* ルーズ&パッシブ
* ルーズ&アグレッシブ
* タイト&パッシブ
* タイト&アグレッシブ
* （JOPT）
* （マイクロステークス）

__**ゲーム性について、以下から選択できる。**__
* MMT
  * 20BB(トナメFT)
  * 30BB(トナメ終盤)
  * 50BB(トナメ中盤)
  * 70BB(トナメ序盤)
  * 100BB(トナメ開始)
* Ring
  * 200BB, レーキ5%, 5BBCAP
  * 100BB, レーキ5%, 5BBCAP

★上記のプリフロップレンジに対して、最もエクスプロイトできるエクスプロイトレンジを設定する
★対象レンジ vs エクスプロイトレンジ のGTOを計算できるようにする。

## 憲章
### やらないこと
* GTO WizerdやPioSolverのような正確な計算をするツールではない
* 細やかなカスタマイズを提供するツールではない
* マルチウェイでの戦い方を提供するツールではない

### 大事なこと
* 90点のGTO精度
* ライト層が、勉強やハンドの振り返りに使える
* 相手がGTO外のレンジである場合、そこからMAXでエクスプロイトするための戦略を勉強できる（★ぶっちゃけこれが一番やりたい）
  * （RustのSolverコードに踏み込んで回収する必要はあるか……？）
  * （でも、コードをみるとレンジ想定も組み込み可能であるから、Exploitavilityや表層コードの修正だけでなんとかなりそうな気がしている）
  * ★問題は、そのエクスプロイト対象のレンジ構成やプレイシミュレーションをどのように再現するか。
    * まず思いつくのはSnowieみたいに沢山のハンドからの機械学習を行うこと。
    * だけど、そもそもその大量のハンド履歴データがないし、ハンドをプレイしている人のラベリングもできていない。
    * だれかいいデータ持ってないかな……

# 動作環境
Web上から利用可能。
アプリでの作成はいったん考えないが、拡張できるような技術選定を行うこと。
(wasmの一部の技術では、アプリ上でマルチスレッドが実現できないとかなんとか)

# バックエンドサービス
ログイン必須にする。
* 過去の計算結果を参照できる。
* バックエンドのキャッシュを利用できる。
* 他人と共有＆ポスト＆議論できる。
* 要望があり、サーバーリソースに余裕があれば、サーバーリソースに計算を依頼することもできる。

## 貢献
* （計算結果のサーバー送信）を必須にする？
  * サーバーに送信できないと、履歴を保存できない。くらいにしておけばよさそう。
    * ログインすれば、履歴や他人のキャッシュ、投稿が視れる。
    * ログインしなければ、単なるフロントエンドのツール。

* off-lineでは使用できないようにする。
* サーバー側で結果の正確性が確保できて初めて、フロント側に結果を保存する。
  * サーバーが落ちてたらさすがにそれは無効でよさそう。。。




# 疑問メモ
* チャンスとはどのような概念か？ -> 純粋にカードがめくられること。


# 関数とリソースの設計情報
## Input
* カード関連
  * hero_range（ルース、タイト、標準、など）
  * villain_range（ルース、タイト、標準、など）
  * flop_cards
* ゲームポジション関連
  * hero_pos
  * villain_pos
* ポット関連
  * xbet_pot
  * pot_bb
  * effective_bb
* 条件面
  * valid_bet_sizes: list
  * villain_movement_trends（相手の特徴。ブラフ過多、過小、など。）

# 20250720 実装メモ
tomlファイルの以下の部分について。
targetをコメントアウトしないと動作しなかった。理由は本当に謎。
```
[package.metadata.component]
package = "holdem-solver:solver"
target = "holdem-solver:solver@0.1.0"
```

# 20250726 実装メモ
tomlファイルについて
* crate-type = ["cdylib"] 
  * 「このクレートは共有ライブラリとして出力すべきだ」という明示。Rustのデフォルトでは lib クレートは Rust専用（rlib）形式でビルドされ、.wasm は生成されない。
* clang, wasmtimeを依存関係に追加していると、Cコンパイラを要求される模様。
  * cargo tree で見ると、どこに書いてあるかがよくわかる。

## Wasm総合メモ
ようやくテストコードをTypescriptから実行できたので、全体像をまとめる。
* Rust側
  * wit-bindgen-rt を使用する
  * コード内での記述
    * witファイルの定義に従って読み込む。
      * `use bindings::exports::holdem_solver::host::my_host::Guest;`
    * Implしたコンポーネントは`bindings::export!(MyFunction with_types_in bindings);`のようにエクスポートする（WITの定義に従う）
  * ビルドする
    * cargo component build (-r) --target wasm32-wasip2
    * target/wasm32-wasip2/debug(release)/ 配下にwasmファイルが作成される。
* Typescript側
  * Wasmファイルのままでは使えないので、Typescript用にデコードする
    * `npm install @bytecodealliance/jco` 
    * `npx @bytecodealliance/jco transpile <wasm file> -o <output path>`
  * 普通のライブラリのように読みだす
    * `import { <Interface name (CamelCase)> } from '<ts file path>'`
    * `interfaceName.functionName()`

# 20250804 メモ
* RustからWasmを読み込もうとした
  * 本ではうまく言っていたけれど、真似してもうまくいかない。
    * どうやら、wasm32-unknown-unknownとwasm32-wasip2 の違いが問題ラシイ。
    * wasip2の場合、内部で余計なパッケージをインストールしており、それがLinkerに対応していないとかなんとか
    * wasip2が不安定と言う話も聞くし、本格的に使っていっていいのかどうか……？
* ひとまず横に置いておいて、GTOToolの実装を薦めよう。
  * テストは、Rustのテスト関数の時点で実施し、Wasmにビルドするのは最後にしよう。
  * その後も、TypescriptやWasmtime、Webappからの呼び出しで行うことにしよう。
    * （結局最後には必要になるけれど、先送りにできることもありそう）
* clang存在しない問題が再燃。キャッシュ消せばいけるかとおもいきや、cargo clearn を実行しても効果なし。
  * 前はなぜできたんだ……？


# 20250818 実装メモ
* `binfgen.export!(***)` に関して、1つのファイル（かな？）から2つ以上Exportすると、2つ目のExportが重複してエラーになるので注意
```
interface game-manager {
  resource game-resource {
    new: static func(number: u32) -> game-resource;
    write: func(number: u32);
    read: func() -> u32;
    up: func() -> u32;
    down: func() -> u32;
  }
}
```
* 上記のようなWITを作成した場合、
  * impl game_manager::Guest for MyGame
  * impl game_manager::GuestGameResource for MyGame
  * の2つを実装する必要がある
* というより、実際にbinding.rsを見ながら実装する方が100倍よい

