# Документация по протоколу `Galaxy`

**В документации будет частично использоваться английский язык, дабы лучше выразить некоторые вещи и термины, если хочешь исправить - PR**.

## Схема работы

Как работает протокол и, соответственно, сам сервер, может быть несколько неочевидно, поэтому считаю нужным описать это. Будет описан успешный маршрут, не включающий в себя возможные ошибки:
1. Создание сервера
- Удаленный клиент подключается к удаленному серверу `Neogrok`, назовем их `клиент` и `сервер` соответственно
- `клиент` просит создать прокси с определенными настройками
- `сервер` запускает другой сервер, назовем его `прокси-сервер` и отсылает информацию о нем `клиент`у

2. Коммуникация
- `сервер` посылает `клиент`у уведомление о подключении к `прокси-сервер`у, присваивает подключившемуся идентификатор, назовем подключившегося клиента `прокси-клиент`.
- `клиент` подключается к локальному серверу и присваивает идентификатор, полученный от сервера, подключению
- `сервер` и `клиент` уведомляют друг друга о записи в сокет, включая при этом идентификатор записавшего `прокси-клиент`а
- `сервер` и `клиент` уведомляют друг друга о закрытии соединения в сокете.


## Структура пакета

Все числа в протоколе кодируются в `little endian` пока не сказано иначе.

Каждый пакет состоит из всего двух полей: `type` и `payload`:

| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| type     |  Packet type with flags | u8          |
| payload  |    Payload              | Unspecified |

payload зависит от типа пакета, тип пакета кодируется так([0; 7] от младшего к старшему):

|7|6|5|4|3|2| 1|0|
|-|-|-|-|-|-|-|-|
|t|t|t|t|t|с|sc|s|

- `t` - тип пакета, беззнаковое число размером 5бит
- `sc` - флаг short client
- `s` - флаг short

Выделить тип и флаги можно так:
```rust
let flags = packed & 0b111;
let type_ = packed >> 3;
```

## Типы

- Error (**0x0**)

| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| value    |    Error code           | u8          |

| Value |    Meaning      |
|-------|:---------------:|
| 0x00  | Unknown command |
| 0x01  | Unsupported     |
| 0x02  | Unimplemented   |
| 0x03  | Access denied   |
| 0x04  | Failed to bind requested address |
| 0x05  | Server was not created |
| 0x06  | Client does not exists |
| 0x07  | Failed to decompress |
| 0x08  | Too long buffer |

**NOTES:**

1. 0x08 и 0x07 критические ошибки, после их получения сервер Вас отключит.

- Ping (**0x01**)

Sent by `server`:
| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| c_algo   | Compression algorithm   | u8          |
| c_level  | Compression level       | u8          |
| buffer   | Read buffer capacity    | u16         |
| len      | Length of string        | u8          |
| srv_name | Name of server(UTF8)    | Unspecified |

**c_algo** can be:

| Value |    Meaning      |
|-------|:---------------:|
| 0x00  | ZSTD            |

Sent by `client`: **No additional params**

- CreateServer (**0x02**)

Sent by `client`:
| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| port     | Port to bind            | u16         |

**NOTES**:

1. Если `port` = 0, то сервер выберет любой свободный
2. Протокол определяется наличием одного из флагов: `c` - TCP, `s` - UDP, `sc` - HTTP, **если ни один флаг не был выбран, то сервер выберет протокол TCP**

Sent by `server`:

| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| c_algo   | Compression algorithm   | u8          |
| c_level  | Compression level       | u8          |

- Connect (**0x03**) && Disconnect (**0x05**)

| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| id       | ID of client            | u8 or u16      |

**NOTES**:
1. Размер поля id определяется наличием флага `sc`, если он есть - размер u8, иначе же - u16

- Forward (**0x04**)

| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| id       | ID of client            | u8 or u16      |
| length   | Length of buffer        | u8 or u16      |
| buffer   | Pile of bytes           | Unspecified |

**NOTES**:
1. Размер поля id определяется наличием флага `sc` по тому же правилу, что и для `Connect`
2. Размер поля length определяется наличием флага `s`, если флаг есть - u8, иначе же - u16
3. buffer может быть сжат алгоритмом сжатия, если он сжат, то проставлен флаг `c`

- AuthorizePassword (**0x06**)

| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| len      | length of password in bytes     | u8          |
| password |     UTF8 String        | Unspecified |

- UpdateRights (**0x07**)

| Field    |      Description        |    Size     |
|----------|:-----------------------:|:-----------:|
| rights   | Rights bits             | u8          |

rights это коллекция битфлагов, маски для которых:
```rust
const CAN_CREATE_TCP      = 1 << 0;
const CAN_SELECT_TCP_PORT = 1 << 1;

const CAN_CREATE_UDP      = 1 << 2;
const CAN_SELECT_UDP_PORT = 1 << 3;

const CAN_CREATE_HTTP     = 1 << 4;
```
