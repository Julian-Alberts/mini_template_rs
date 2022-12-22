# Default Modifiers

## add
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Adds two numbers
### Arguments
| argument | description |  type  | Default | Nullable |
|:--------:|:-----------:|:------:|:-------:|:--------:|
|    a     |   Number    | Number |    -    |    No    |
|    b     |   Number    | Number |    -    |    No    |

Returns Number

## div
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Divides two numbers
### Arguments
| argument | description |  type  | Default | Nullable |
|:--------:|:-----------:|:------:|:-------:|:--------:|
|    a     |   Number    | Number |    -    |    No    |
|    b     |   Number    | Number |    -    |    No    |

Returns Number

## lower
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Converts a string to lower case
### Arguments
| argument | description |  type  | Default | Nullable |
|:--------:|:-----------:|:------:|:-------:|:--------:|
|  input   |   String    | String |    -    |    No    |


Returns String

## match
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |      regex       |        yes        |

Checks if a regex matches to given string. If group is `0` the entire regex 
must match.

### Arguments
| argument |  description   |  type  | Default | Nullable |
|:--------:|:--------------:|:------:|:-------:|:--------:|
|  input   |     String     | String |    -    |    No    |
|  regex   | regex to match | String |    -    |    No    |
|  group   | Group to match | Number |    0    |   Yes    |

Returns: Boolean

## mul
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Multiplies two numbers
### Arguments
| argument | description |  type  | Default | Nullable |
|:--------:|:-----------:|:------:|:-------:|:--------:|
|    a     |   Number    | Number |    -    |    No    |
|    b     |   Number    | Number |    -    |    No    |

Returns Number

## repeat
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Repeats a string n times
### Arguments
| argument | description |  type  | Default | Nullable |
|:--------:|:-----------:|:------:|:-------:|:--------:|
|  input   |   String    | String |    -    |    No    |
|    n     |   Number    | Number |    -    |    No    |

Returns String

## replace
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Replaces a section inside a string
### Arguments
| argument |     description      |  type  | Default | Nullable |
|:--------:|:--------------------:|:------:|:-------:|:--------:|
|  input   |        String        | String |    -    |    No    |
|   from   | Needle to search for | String |    -    |    No    |
|    to    |     Replacement      | String |    -    |    No    |
|  count   | maximum replacements | Number |    0    |   yes    |

Returns String

## replace
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |      regex       |        yes        |

Replaces a section inside a string
### Arguments
| argument |     description      |  type  | Default | Nullable |
|:--------:|:--------------------:|:------:|:-------:|:--------:|
|  input   |        String        | String |    -    |    No    |
|  regex   |   Regex to replace   | String |    -    |    No    |
|    to    |     Replacement      | String |    -    |    No    |
|  count   | maximum replacements | Number |    0    |   yes    |

Returns String

## slice
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Returns string slice with a given length
### Arguments
| argument |     description     |  type  | Default | Nullable |
|:--------:|:-------------------:|:------:|:-------:|:--------:|
|  input   |       String        | String |    -    |    No    |
|  start   | begin of the slice  | Number |    -    |    No    |
|  length  | length of the slice | Number |    -    |    No    |

Returns String

## sub
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Subtracts two numbers
### Arguments
| argument | description |  type  | Default | Nullable |
|:--------:|:-----------:|:------:|:-------:|:--------:|
|    a     |   Number    | Number |    -    |    No    |
|    b     |   Number    | Number |    -    |    No    |

Returns Number

## upper
| since | requires feature |  default feature  |
|-------|:----------------:|:-----------------:|
| 0.1.0 |        -         |        yes        |

Converts a string to upper case
### Arguments
| argument | description |  type  | Default | Nullable |
|:--------:|:-----------:|:------:|:-------:|:--------:|
|  input   |   String    | String |    -    |    No    |


Returns String