use serde::{Deserialize, Serialize};
use yew::prelude::*;
use gloo_net::websocket::{futures::WebSocket, Message};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::HtmlInputElement;
use crate::User;
use gloo_timers::future::TimeoutFuture;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    data_array: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    data: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Clone, Debug, PartialEq)]
struct UserProfile {
    name: String,
    avatar: String,
}

#[function_component(Chat)]
pub fn chat() -> Html {
    let user_ctx = use_context::<UseStateHandle<User>>().expect("No context found");
    let username = user_ctx.username.clone();

    let users = use_state(Vec::<UserProfile>::new);
    let chat_input = use_state(String::new);

    let messages = use_mut_ref(Vec::<MessageData>::new);
    let render_trigger = use_state(|| 0);

    type WsSender = Rc<RefCell<SplitSink<WebSocket, Message>>>;
    let ws_sender = use_state(|| None::<WsSender>);

    {
        let users = users.clone();
        let messages = messages.clone();
        let render_trigger = render_trigger.clone();
        let ws_sender = ws_sender.clone();
        let username = username.clone();

        use_effect_with((), move |_| {
            let ws = WebSocket::open("ws://localhost:8080").expect("Gagal membuka WebSocket");
            let (mut write, mut read) = ws.split();

            let register_msg = WebSocketMessage {
                message_type: MsgTypes::Register,
                data: Some(username),
                data_array: None,
            };

            wasm_bindgen_futures::spawn_local(async move {
                TimeoutFuture::new(500).await;
                let json_str = serde_json::to_string(&register_msg).unwrap();
                let _ = write.send(Message::Text(json_str)).await;
                
                ws_sender.set(Some(Rc::new(RefCell::new(write))));
            });

            wasm_bindgen_futures::spawn_local(async move {
                while let Some(msg) = read.next().await {
                    if let Ok(Message::Text(text)) = msg {
                        if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                            match ws_msg.message_type {
                                MsgTypes::Users => {
                                    if let Some(user_list) = ws_msg.data_array {
                                        let profiles = user_list.into_iter()
                                            .filter(|name| !name.trim().is_empty()) 
                                            .map(|name| UserProfile {
                                                avatar: format!("https://api.dicebear.com/7.x/adventurer/svg?seed={}", name),
                                                name,
                                            }).collect();
                                        users.set(profiles);
                                    }
                                }
                                MsgTypes::Message => {
                                    if let Some(msg_data_str) = ws_msg.data {
                                        if let Ok(msg_data) = serde_json::from_str::<MessageData>(&msg_data_str) {
                                            messages.borrow_mut().push(msg_data);
                                            render_trigger.set(*render_trigger + 1);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            });

            || ()
        });
    }

    let oninput = {
        let chat_input = chat_input.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            chat_input.set(input.value());
        })
    };

    let onsubmit = {
        let chat_input = chat_input.clone();
        let ws_sender = ws_sender.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let msg_text = (*chat_input).clone();
            if msg_text.is_empty() { return; }

            if let Some(sender_rc) = &*ws_sender {
                let msg_obj = WebSocketMessage {
                    message_type: MsgTypes::Message,
                    data: Some(msg_text),
                    data_array: None,
                };
                let json_str = serde_json::to_string(&msg_obj).unwrap();
                
                let sender_rc = Rc::clone(sender_rc);
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = sender_rc.borrow_mut().send(Message::Text(json_str)).await;
                });
            }
            chat_input.set(String::new());
        })
    };
    
    html! {
        <div class="flex w-screen h-screen bg-white">
            <div class="flex-none w-64 h-screen bg-gray-100 border-r border-gray-300 flex flex-col">
                <div class="text-xl p-4 font-bold border-b border-gray-300 bg-white">{"Users"}</div>
                <div class="overflow-y-auto grow p-2 space-y-2">
                    { for users.iter().map(|u| html! {
                        <div class="flex items-center bg-white rounded p-2 shadow-sm border border-gray-200">
                            <img class="w-10 h-10 rounded-full bg-gray-200" src={u.avatar.clone()} />
                            <div class="ml-3 font-medium text-gray-800">{u.name.clone()}</div>
                        </div>
                    }) }
                </div>
            </div>

            <div class="grow h-screen flex flex-col">
                <div class="w-full h-14 border-b-2 border-gray-300 flex items-center px-4 font-bold text-gray-700 bg-gray-50">
                    {"💬 YewChat"}
                </div>
                
                <div class="w-full grow overflow-y-auto p-4 flex flex-col space-y-4 bg-gray-50">
                    { for messages.borrow().iter().map(|m| {
                        let is_me = m.from == username;
                        let avatar = users.iter()
                            .find(|u| u.name == m.from)
                            .map(|u| u.avatar.clone())
                            .unwrap_or_else(|| format!("https://api.dicebear.com/7.x/adventurer/svg?seed={}", m.from));

                        html! {
                            <div class={format!("flex items-end w-3/4 {}", if is_me { "self-end flex-row-reverse" } else { "" })}>
                                <img class={format!("w-8 h-8 rounded-full bg-gray-200 shadow-sm {}", if is_me { "ml-3" } else { "mr-3" })} src={avatar} />
                                <div class={format!("p-3 rounded-lg shadow-sm {}", if is_me { "bg-blue-500 text-white" } else { "bg-white text-gray-800 border border-gray-200" })}>
                                    <div class={format!("text-xs font-bold mb-1 {}", if is_me { "text-blue-100" } else { "text-gray-500" })}>
                                        {m.from.clone()}
                                    </div>
                                    <div class="text-sm break-words">
                                        if m.message.ends_with(".gif") {
                                            <img class="mt-2 rounded" src={m.message.clone()} />
                                        } else {
                                            {m.message.clone()}
                                        }
                                    </div>
                                </div>
                            </div>
                        }
                    }) }
                </div>

                <form onsubmit={onsubmit} class="w-full h-16 border-t-2 border-gray-300 flex px-4 items-center bg-white">
                    <input 
                        {oninput}
                        value={(*chat_input).clone()}
                        type="text" 
                        placeholder="Type a message..." 
                        class="block grow py-2 pl-4 mx-3 bg-gray-100 rounded-full outline-none focus:ring-2 focus:ring-blue-500" 
                        required=true 
                    />
                    <button type="submit" class="p-2 bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center text-white hover:bg-blue-500 shadow-sm">
                        <svg fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5">
                            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                        </svg>
                    </button>
                </form>
            </div>
        </div>
    }
}