User: {name}
User-ID: {userId}

{i = 0}
{cart_len = cart|len}
{while i < cart_len}
    {cart[i].id}: {cart[i].name}
    {i = i|add:1}
{endwhile}
