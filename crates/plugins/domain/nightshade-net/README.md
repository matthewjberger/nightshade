# nightshade-net

This crate is a topic broker and its clients. A `Broker` accepts connections and routes published frames to whoever subscribed to the topic, and a `Client` connects, subscribes and publishes. The payload stays the caller's, as JSON or as postcard bytes, so the crate moves messages without knowing what they mean.

## Composing it

The crate carries no plugin and no ECS. A program opens a broker and a client on a thread of its own and hands the frames to its game through channels.

## Using it

```rust
use nightshade_net::{Broker, Client, ClientSettings, Message, Reception, Route, run_on_thread};

async fn run(role: Role, bind: String, connect: String) {
    let _broker = if role == Role::Host {
        Broker::start(&bind).await.ok()
    } else {
        None
    };

    let client = Client::new("player", ClientSettings::default());
    client.connect(&connect).await;
    client.subscribe(&["sandbox/state"]).await;
}
```

`Broker::start` binds an address and serves until it is dropped, so the copy that hosts keeps it alive beside its own client. `Client::new` names the client and takes its settings, `connect` dials the broker and reports an `Error` the caller can retry on, and `subscribe` takes the topics this client wants. A published frame arrives as a `Reception` carrying a `Message`, which reads back as JSON with `read_json` or as bytes. `Route::Local` marks a publish the broker keeps to its own subscribers rather than forwarding across a bridge, and `Route::Global` is the default. `run_on_thread` starts the async task on a thread of its own. `BrokerContract` declares the topics the broker itself speaks, among them the peer connected and disconnected reports.

## Taking it from another crate

The crate carries no `contract` feature, because the whole crate is the contract. It takes no `nightshade` dependency at all, so a program or a crate that speaks over the network names it whole.
