{if var1|upper == "HELLO WORLD" && var2 < 10}
    {var1|lower}
{else}
    {var1|replace_regex:"[hH]":"g"}
{endif}
