use reqwasm::Error;
//use reqwest::Client;
use serde::Deserialize;
use web_sys::{HtmlCanvasElement, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
//use yew_components::select::Select;

#[derive(PartialEq, Clone, Properties)]
pub struct SelectProps {
    pub class: String,
    pub on_change: Callback<Option<ItemType>>,
}

#[function_component(SelectItemType)]
pub fn select_rendered_at(props: &SelectProps) -> Html {
    let on_change = {
        let on_change = props.on_change.clone();
        move |e: Event| {
            let input: HtmlSelectElement = e.target_unchecked_into();
            let item_type = match input.value().as_str() {
                "editors" => Some(ItemType::Editors),
                "languages" => Some(ItemType::Langs),
                "projects" => Some(ItemType::Projects),
                _ => None,
            };
            on_change.emit(item_type);
        }
    };
    html! {
        <select
            class={props.class.clone()}
            onchange={on_change}
        >
            <option value="" selected=true>{ "選択してください" }</option>
            <option value="editors">{ "エディタ" }</option>
            <option value="languages">{ "言語" }</option>
            <option value="projects">{ "プロジェクト" }</option>
        </select>
    }
}

#[derive(PartialEq, Clone)]
pub enum ItemType {
    Editors,
    Langs,
    Projects,
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ItemType::Editors => write!(f, "editors"),
            ItemType::Langs => write!(f, "languages"),
            ItemType::Projects => write!(f, "projects"),
        }
    }
}

pub struct SummaryGraph {
    canvas_ref: NodeRef,
    sel_type: Option<ItemType>,
    start_date: String,
    end_date: String,
    render_data: Vec<RankingItem>,
}

#[derive(PartialEq, Debug, Properties, Clone)]
pub struct Props {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct RankingItem {
    title: String,
    hours: f64,
}

pub enum Msg {
    Exec,
    TypeChanged(ItemType),
    GetResult(Vec<RankingItem>),
    StartDateChanged(String),
    EndDateChanged(String),
    Error,
}

impl Component for SummaryGraph {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            sel_type: None,
            start_date: "".into(),
            end_date: "".into(),
            canvas_ref: NodeRef::default(),
            render_data: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Exec => {
                //DialogService::alert(&format!("hoeほげ～: {}", self.id));
                // validation
                if self.sel_type.is_none() || self.start_date.len() == 0 || self.end_date.len() == 0
                {
                    gloo::dialogs::alert("未入力があります");
                    return false;
                }
                crate::show_loading();

                // build url
                let item_type = match self.sel_type.as_ref().unwrap() {
                    ItemType::Editors => "editors",
                    ItemType::Langs => "langs",
                    ItemType::Projects => "projects",
                };

                let req_url = format!(
                    "/wakalog/api/{}/{}/{}",
                    item_type, self.start_date, self.end_date
                );
                ctx.link().send_future(async move {
                    let res = reqwasm::http::Request::get(req_url.clone().as_str())
                        .send()
                        .await;
                    if res.is_err() {
                        return Msg::Error;
                    }
                    let res_json = res.unwrap().json().await;
                    if res_json.is_err() {
                        return Msg::Error;
                    }
                    let res_json = res_json.unwrap();
                    Msg::GetResult(res_json)
                });
            }
            Msg::TypeChanged(itemtype) => {
                //DialogService::alert(&format!("{}", itemtype));
                self.sel_type = Some(itemtype)
            }
            Msg::Error => {
                // 念のため、カバーは外しておく
                crate::hide_loading();
                gloo::dialogs::alert("Error");
            }
            Msg::StartDateChanged(s) => {
                //DialogService::alert(&format!("start_date: {}", s));
                self.start_date = s.replace("-", "");
            }
            Msg::EndDateChanged(s) => {
                //DialogService::alert(&format!("end_date: {}", s));
                self.end_date = s.replace("-", "");
            }
            Msg::GetResult(rank) => {
                crate::hide_loading();
                let canvas_opt: Option<HtmlCanvasElement> =
                    self.canvas_ref.cast::<HtmlCanvasElement>();
                self.render_data = rank;
                return true;
                //DialogService::alert(&format!("nooop: {:?}", rank))
            }
        }
        false
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        //let types = vec![ItemType::Editors, ItemType::Langs, ItemType::Projects];
        html! {
            <section class="section">
                <div class="box">

                    <div class="column">

                        <div class="field">
                            <label class="label">{ "種類" }</label>
                            <div class="control">
                            <SelectItemType
                                class="select"
                                on_change={ctx.link().callback(|ev| {
                                    if let Some(ty) = ev {
                                        Msg::TypeChanged(ty)
                                    } else {
                                        Msg::Error
                                    }
                                })} />
                            </div>
                        </div>

                    </div>
                    <div class="columns">
                        <div class="column">

                            <div class="field">
                                <label class="label">{ "開始日" }</label>
                                <div class="control">
                                <input
                                    class="input"
                                    type="date"
                                    id="start_date"
                                    oninput={ctx.link().callback(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        Msg::StartDateChanged(input.value())
                                    })}
                                />
                                </div>
                            </div>

                        </div>
                        <div class="column">

                            <div class="field">
                                <label class="label">{ "終了日" }</label>
                                <div class="control">
                                <input
                                    class="input"
                                    type="date"
                                    id="end_date"
                                    oninput={ctx.link().callback(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        Msg::EndDateChanged(input.value())
                                    })} />
                                </div>
                            </div>

                        </div>
                    </div>
                    <div class="column">

                        <button class="button is-primary" onclick={ctx.link().callback(|_| Msg::Exec )}>{ "実行" }</button>

                    </div>
                </div>
                <div class="box">
                    { self.make_result() }
                </div>

                <div id="canvas-container">
                    <canvas width=1200 height=800 ref={self.canvas_ref.clone()}>
                    </canvas>
                </div>
            </section>
        }
    }
}

impl SummaryGraph {
    fn make_result(&self) -> Html {
        if self.render_data.len() == 0 {
            html! {
              <div class="notification">{
                  "未検索もしくは検索結果が存在しませんでした。"
              }</div>
            }
        } else {
            html! {
                <table class="table">
                <thead>
                    <tr><td>{ "タイトル" }</td><td>{ "時間" }</td></tr>
                </thead>
                <tbody>
                {
                    for self.render_data.iter().map(|row| {
                        html! {
                            <tr>
                                <td>{ &row.title }</td>
                                <td>{ format!("{:.2} 時間", row.hours) }</td>
                            </tr>
                        }

                    })
                }
                </tbody>
                </table>
            }
        }
    }
}
