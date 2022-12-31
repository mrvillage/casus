# casus

Casus is a simple library containing a handful of useful generic async primitives. At present, it contains `Event` and `Waiter` primitives.

## Event

The Event primitive allows a future to await the completion of an event. Once the event is completed, all futures trying to await it will immediately return and continue until the event is reset.

```rs
use casus::Event;

let event = Event::new();

// this will block until Event::set is called elsewhere
event.wait().await;
```

## Waiter

The Waiter primitive simply waits to be woken up with it's return value.

```rs
use casus::Waiter;

let waiter = Waiter::new();

// this will block until Event::wake is called elsewhere
waiter.await;
```
