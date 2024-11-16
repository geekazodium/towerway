# Tower Defense + Conway Game of Life

Game is available on itch at https://pearstar.itch.io/child-of-entropy[https://pearstar.itch.io/child-of-entropy]

## Development Setup:

rust library comes compiled

## CSharp Access of Rust Library

accessing a Rust library Value:
```
this.Call("get_<name of exported value>").as<type of value>(); //-> __value__
```

```
this.Call("set_<name of exported value>", new Variant[]{Variant.From<type of value>( __value__ )});
```
