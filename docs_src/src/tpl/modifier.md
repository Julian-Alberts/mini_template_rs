# Modifier
Modifiers allow you to modify values before they are used or printed. They 
are allowed everywhere where you can use values except for modifier arguments.
An example call to a modifier looks like this. 
```
{my_var|replace:"Foo":"Bar"}
```
This would replace every occurrence of `Foo` with `Bar`. Modifiers may have
optional modifiers. For example `replace` supports an additional `count` 
argument. If you wanted tho only replace two instances of `Foo` you would write 
it like is.
```
{my_var|replace:"Foo":"Bar":2}
```
Modifiers can be chained for more complex behavior. Modifiers will always be 
executed from left to right. If you need to use the result of a modifier as 
an argument for another modifier you have to [assign](assign.md) it to a variable first.