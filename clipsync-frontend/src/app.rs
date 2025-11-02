use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let (host_address, write_host_address) = signal(String::new());
    let (passkey, write_passkey) = signal(String::new());
    let (logs, write_logs) = signal(vec![
        "Program started.".to_string(),
        "Waiting for connection...".to_string(),
    ]);

    let connect = move |_| {
        write_logs.update(|l| l.push(format!("Connecting to {}...", host_address.get())));
    };

    let reconnect = move |_| {
        write_logs.update(|l| l.push("Reconnecting...".to_string()));
    };

    view! {
        <div class="min-h-screen bg-gray-100 flex flex-col items-center justify-center p-4">
            <div class="bg-white shadow-md rounded p-6 w-full max-w-md">
                <h1 class="text-2xl font-bold mb-4 text-center">"Clipboard Sync"</h1>

                <div class="mb-4">
                    <label class="block text-gray-700 mb-1" for="host">"Host Address"</label>
                    <input
                        id="host"
                        type="text"
                        class="w-full border border-gray-300 rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        placeholder="e.g. ws://192.168.0.100:9000"
                        on:input=move |ev| write_host_address.set(event_target_value(&ev))
                        prop:value=host_address
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 mb-1" for="passkey">"Passkey (Optional)"</label>
                    <input
                        id="passkey"
                        type="password"
                        class="w-full border border-gray-300 rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        placeholder="Optional passkey"
                        on:input=move |ev| write_passkey.set(event_target_value(&ev))
                        prop:value=passkey
                    />
                </div>

                <div class="flex justify-between mb-4">
                    <button
                        class="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded"
                        on:click=connect
                    >
                        "Connect"
                    </button>
                    <button
                        class="bg-yellow-500 hover:bg-yellow-600 text-white font-semibold py-2 px-4 rounded"
                        on:click=reconnect
                    >
                        "Reconnect"
                    </button>
                </div>

                <div class="bg-gray-50 border border-gray-300 rounded p-3 h-40 overflow-y-scroll text-sm text-gray-700">
                    {move || logs.get().iter().map(|log| view! {
                        <div class="leading-tight">{log.clone()}</div>
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}
