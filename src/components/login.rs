use crate::{Route, User};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Login)]
pub fn login() -> Html {
    let user_ctx = use_context::<UseStateHandle<User>>().expect("No context found");

    let navigator = use_navigator().unwrap();

    let input_val = use_state(String::new);

    let oninput = {
        let input_val = input_val.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input_val.set(input.value());
        })
    };

    let onsubmit = {
        let input_val = input_val.clone();
        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            user_ctx.set(User {
                username: (*input_val).clone(),
            });

            navigator.push(&Route::Chat);
        })
    };

    html! {
        <form onsubmit={onsubmit} class="flex flex-col gap-4 bg-gray-800 p-8 rounded-lg shadow-lg w-80">
            <h2 class="text-white text-2xl font-bold text-center mb-2">{"Join YewChat"}</h2>
            <input
                {oninput}
                class="rounded p-3 outline-none text-gray-800 focus:ring-2 focus:ring-blue-500"
                placeholder="Enter Username"
                value={(*input_val).clone()}
                required=true
            />
            <button
                type="submit"
                disabled={input_val.len() < 2}
                class="bg-blue-600 text-white rounded p-3 font-semibold transition-colors disabled:bg-gray-600 hover:bg-blue-500"
            >
                {"Connect"}
            </button>
        </form>
    }
}
