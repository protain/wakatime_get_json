use std::str::FromStr;

use serde::Deserialize;
use web_sys::HtmlCanvasElement;
use yew::{
    format::Nothing,
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        ConsoleService, DialogService, FetchService,
    },
};
use yew_components::select::Select;

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
    id: u64,
    fetch_task: Option<FetchTask>,
    sel_type: Option<ItemType>,
    start_date: String,
    end_date: String,
    render_data: Vec<RankingItem>,
    link: ComponentLink<Self>,
}

#[derive(Debug, Properties, Clone)]
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            id: props.id,
            fetch_task: None,
            sel_type: None,
            start_date: "".into(),
            end_date: "".into(),
            canvas_ref: NodeRef::default(),
            render_data: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Exec => {
                //DialogService::alert(&format!("hoeほげ～: {}", self.id));
                // validation
                if self.sel_type.is_none() || self.start_date.len() == 0 || self.end_date.len() == 0
                {
                    DialogService::alert("未入力があります");
                    return false;
                }
                unsafe {
                    crate::show_loading();
                }

                // build url
                let item_type = match self.sel_type.as_ref().unwrap() {
                    ItemType::Editors => "editors",
                    ItemType::Langs => "langs",
                    ItemType::Projects => "projects",
                };
                let get_request = Request::get(&format!(
                    "/wakalog/api/{}/{}/{}",
                    item_type, self.start_date, self.end_date
                ))
                .body(Nothing)
                .expect("Failed to build request.");
                let callback =
                    self.link
                        .callback(|response: Response<Result<String, anyhow::Error>>| {
                            let body = String::from_str(response.body().as_ref().unwrap()).unwrap();
                            ConsoleService::info(&format!(
                                "response stat: {:?}, body: {:?}",
                                response.status(),
                                body
                            ));
                            if response.status().is_success() {
                                Msg::GetResult(
                                    serde_json::from_str::<Vec<RankingItem>>(&body).unwrap(),
                                )
                            } else {
                                Msg::Error
                            }
                        });
                let res =
                    FetchService::fetch(get_request, callback).expect("failed to send request");
                self.fetch_task = Some(res);
            }
            Msg::TypeChanged(itemtype) => {
                //DialogService::alert(&format!("{}", itemtype));
                self.sel_type = Some(itemtype)
            }
            Msg::Error => {
                // 念のため、カバーは外しておく
                unsafe {
                    crate::hide_loading();
                }
                DialogService::alert("Error")
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
                unsafe {
                    crate::hide_loading();
                }
                let canvas_opt: Option<HtmlCanvasElement> =
                    self.canvas_ref.cast::<HtmlCanvasElement>();
                self.render_data = rank;
                return true;
                //DialogService::alert(&format!("nooop: {:?}", rank))
            }
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let types = vec![ItemType::Editors, ItemType::Langs, ItemType::Projects];
        html! {
            <section class="section">
                <div class="box">

                    <div class="column">

                        <div class="field">
                            <label class="label">{ "種類" }</label>
                            <div class="control">
                            <Select<ItemType>
                                class="select"
                                options=types
                                on_change=self.link.callback(|ev| Msg::TypeChanged(ev)) />
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
                                    oninput=self.link.callback(|e: InputData| {
                                        Msg::StartDateChanged(e.value)
                                    })
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
                                    oninput=self.link.callback(|e: InputData| Msg::EndDateChanged(e.value)) />
                                </div>
                            </div>

                        </div>
                    </div>
                    <div class="column">

                        <button class="button is-primary" onclick=self.link.callback(|_| Msg::Exec )>{ "実行" }</button>

                    </div>
                </div>
                <div class="box">
                    { self.make_result() }
                </div>

                <div id="canvas-container">
                    <canvas width=1200 height=800 ref=self.canvas_ref.clone()>
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
