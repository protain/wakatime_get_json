# wakatime_get_json

wakatime (https://wakatime.com/) の情報をコマンドラインでアクセスして情報を取得しローカルファイルに残すためのものです。
使用にあたってはAPI-KEYといった文字列が必要です。
https://wakatime.com/developers#authentication のUsing API Keyの「API key」のリンクで表示されるキー文字列や
APIを作成した時の設定画面に表示される api-key / secret が必要です。

--

Yew 0.19へのマイグレーションについて

- ComponentのtraitのI/F変更

Context<Self> に全般的に置き換え。

  - 旧: create(Self::Properties, ComponentLink<Self>) -> Self
  - 新: create(&Context<Self>) -> Self

  - 旧: update<&mut self, Self::Message> -> ShouldRender
  - 新: update<&mut self, Context<Self>, Self::Message> -> bool

  - 旧: view(&self) -> Html
  - 新: view(&self, &Context<Self>) -> Html

  - 新: changed(&mut self, &Context<Self>) -> bool
  - 新: rendered(&mut self, &Context<Self>, bool)
  - 新: destroy(&mut self, &Context<Self>) {}

- ComponentLinkの廃止

Context<Self>経由に置き換える。
Context<Self> の link() が代替えになる。
プロパティは Context<Self> の props()が代替えになる。

- Routerの破壊的変更

全く別物に近くなった。

- DialogServiceの廃止

glooクレートへの置き換え

- FetchServiceの廃止

reqwasmへの置き換え
→非同期呼び出しなので非同期ブロックを使用する。
