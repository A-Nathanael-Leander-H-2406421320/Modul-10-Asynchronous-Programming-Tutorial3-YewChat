mod components;

use components::chat::Chat;
use components::login::Login;

use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Login,
    #[at("/chat")]
    Chat,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Debug, PartialEq, Clone)]
pub struct User {
    pub username: String,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Login => html! { <Login /> },
        Route::Chat => html! { <Chat /> },
        Route::NotFound => html! { <h1 class="text-white text-3xl">{"404 Not Found"}</h1> },
    }
}

#[function_component(App)]
fn app() -> Html {
    let user_state = use_state(|| User {
        username: String::new(),
    });

    html! {
        <ContextProvider<UseStateHandle<User>> context={user_state}>
            <BrowserRouter>
                <div class="flex w-screen h-screen bg-gray-900 items-center justify-center">
                    <Switch<Route> render={switch} />
                </div>
            </BrowserRouter>
        </ContextProvider<UseStateHandle<User>>>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
