This is a minimal reproduction of dom slot error in Yew probably related to unstable rendering order.

To run the example:

```bash
trunk build
```

and then

```bash
PORT=8080 cargo run -p backend
```

The website will panic with the following error:

```
 INFO scheduler_loop:render{component.id=9}: frontend::app: Rendering edge from NodeIndex(2) to NodeIndex(0)

 INFO scheduler_loop:render{component.id=9}: frontend::app: Rendering edge from NodeIndex(1) to NodeIndex(2)

 INFO scheduler_loop:render{component.id=9}: frontend::app: Rendering edge from NodeIndex(0) to NodeIndex(1)

 INFO scheduler_loop:render{component.id=9}: frontend::app: Rendering edge from NodeIndex(1) to NodeIndex(2)

 INFO scheduler_loop:render{component.id=9}: frontend::app: Rendering edge from NodeIndex(2) to NodeIndex(0)

 INFO scheduler_loop:render{component.id=9}: frontend::app: Rendering edge from NodeIndex(0) to NodeIndex(1)

panicked at /home/maa/.cargo/git/checkouts/yew-07f42ef43f99cbca/50f987d/packages/yew/src/dom_bundle/position.rs:114:13:
Should not use a trapped DomSlot. Please report this as an internal bug in yew.
```

The `HomePage` component renders three `<rect/>` elements forming a triangle. The logs shows that it renders twice with different orders, which might be the reason for the panic.

The `stable-order` branch contains a version that simplifies the iterator creation in the HomePage component and doesn't trigger the error.

