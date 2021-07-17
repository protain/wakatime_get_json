use yew::{
    format::Nothing,
    prelude::*,
    services::{
        fetch::{Request, Response},
        DialogService, FetchService,
    },
};

pub struct SummaryGraph {
    link: ComponentLink<Self>,
    id: u64,
}

#[derive(Debug, Properties, Clone)]
pub struct Props {
    pub id: u64,
}

pub enum Msg {
    HoeHoe,
    Noop,
    Error,
}

impl Component for SummaryGraph {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, id: props.id }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::HoeHoe => {
                DialogService::alert(&format!("hogeほげ～: {}", self.id));
                let get_request = Request::get("http://localhost:5005/editors/20200101/20201231")
                    .body(Nothing)
                    .expect("Failed to build request.");
                let callback =
                    self.link
                        .callback(|response: Response<Result<String, anyhow::Error>>| {
                            if response.status().is_success() {
                                Msg::Noop
                            } else {
                                Msg::Error
                            }
                        });
                let _ = FetchService::fetch(get_request, callback).unwrap();
            }
            Msg::Noop => DialogService::alert(&format!("noop")),
            Msg::Error => DialogService::alert("Error"),
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <button onclick=self.link.callback(|_| Msg::HoeHoe )>
                { "ほえほえ～l" }
                </button>
            </>
        }
    }
}
