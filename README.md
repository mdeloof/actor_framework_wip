# Armature

Armature is an event-driven stateful actor framework for Rust, enabling you to design complex systems that are easy to reaseon about

## What does that mean?

- **Event-driven:** events are the driving force behind any change to the 
system state. Events are gathered in a queue and dispatched to the actors.
- **Stateful actor:** every actor contains a hierarchical state machine 
that processes incoming events.

See the example for how to use.