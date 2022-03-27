# Conditional 
Conditional statements start with `{if}` and end with `{endif}`. The content between the two
delimiters only gets rendered if the condition is evaluated to true.

```
{condition = true}
{if condition}
    Foo
{endif}
```
Returns:
```
Foo
```

You can also add an `{else}`. Its content will be returned if the condition is false.

```
{condition = false}
{if condition}
    Foo
{else}
    Bar
{endif}
```
Returns:
```
Bar
```

