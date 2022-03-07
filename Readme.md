# `fe2o3-amqp`

An implementation of the AMQP1.0 protocol based on serde and tokio.

## Quick start

Below is an example with a local broker (
[`TestAmqpBroker`](https://github.com/Azure/amqpnetlite/releases/download/test_broker.1609/TestAmqpBroker.zip))
listening on the localhost. The broker is executed with the following command

```powershell
./TestAmqpBroker.exe amqp://localhost:5672 /creds:guest:guest /queues:q1
```

The following code requires the [`tokio`] async runtime added to the dependencies.

```rust
use fe2o3_amqp::{Connection, Session, Sender, Receiver};

#[tokio::main]
async fn main() {
    let mut connection = Connection::open(
        "connection-1",                     // container id
        "amqp://guest:guest@localhost:5672" // url
    ).await.unwrap();

    let mut session = Session::begin(&mut connection).await.unwrap();

    // Create a sender
    let mut sender = Sender::attach(
        &mut session,           // Session
        "rust-sender-link-1",   // link name
        "q1"                    // target address
    ).await.unwrap();

    // Create a receiver
    let mut receiver = Receiver::attach(
        &mut session,           // Session
        "rust-receiver-link-1", // link name
        "q1"                    // source address
    ).await.unwrap();

    // Send a message to the broker
    sender.send("hello AMQP").await.unwrap();

    // Receive the message from the broker
    let delivery = receiver.recv::<String>().await.unwrap();
    receiver.accept(&delivery).await.unwrap();

    // Detach links with closing Detach performatives
    sender.close().await.unwrap();
    receiver.close().await.unwrap();

    // End the session
    session.end().await.unwrap();

    // Close the connection
    connection.close().await.unwrap();
}
```

## Components

| Name | Description |
|------|-------------|
|`serde_amqp_derive`| Custom derive macro for described types as defined in AMQP1.0 protocol |
|`serde_amqp`| AMQP1.0 serializer and deserializer as well as primitive types |
|`fe2o3-amqp-types`| AMQP1.0 data types |
|`fe2o3-amqp`| Implementation of AMQP1.0 `Connection`, `Session`, and `Link` |
|`fe2o3-amqp-ext`| Extension types and implementations |

## Road map

- [x] Proper error handling (more or less)
- [ ] Pipelined open
- [ ] SASL SCRAM-SHA1
- [ ] Transaction
- [ ] Listeners
- [ ] Link re-attachment
- [ ] Message batch disposition
