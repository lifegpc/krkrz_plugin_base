# httpreq

An asynchronous HTTP client plugin example for Kirikiri Z (`krkrz`), built with `krkrz_plugin_base`, `tokio`, and `reqwest`. All callbacks are dispatched back onto the Kirikiri Z window thread via `AsyncExecutor`.

## Loading the Plugin

```tjs
Plugins.link("httpreq.dll");
```

Linking the plugin registers `HttpReq` and `AbortSignal` in the TJS global scope.

---

## API Reference

### `HttpReq`

#### `HttpReq.get(url, config, callback, onerror)`

Sends an asynchronous HTTP `GET` request.

- **`url`** (`String`): The request URL.
- **`config`** (`Dictionary` | `void`): Optional configuration dictionary:
  - **`response_type`** (`String`, optional): How to decode the response body. Supported values:
    - `"text"` *(default)*: Populates `response.text` as a `String`.
    - `"bytes"`: Populates `response.bytes` as an `Octet`.
    - `"json"`: Parses the body as JSON and populates `response.json` as a TJS value (`Dictionary`, `Array`, `String`, `Integer`, `Real`, or `void`).
  - **`no_error_handle`** (`Boolean`, optional, default: `false`):
    - When `false`, non-2xx HTTP status codes trigger `onerror`.
    - When `true`, non-2xx HTTP responses are still passed to `callback` (with `response.ok == false`).
  - **`signal`** (`AbortSignal` | `void`, optional): An `AbortSignal` instance used to abort the request.
- **`callback`** (`Function(response)`): Called on the window thread when the request succeeds.
- **`onerror`** (`Function(error)` | `void`, optional): Called on the window thread if the request fails, JSON parsing fails, or the request is aborted.
  - On network/HTTP/parse errors, `error` is an error message `String`.
  - When aborted via `AbortSignal`, `error` is `signal.reason` (or `"Aborted"` if no reason was passed to `signal.abort()`).

#### Response Object (`SimpleResponse`)

The `response` dictionary passed to `callback` contains:

| Field | Type | Description |
| --- | --- | --- |
| `ok` | `Boolean` | `true` (`1`) if `code` is in `200..=299`, `false` (`0`) otherwise. |
| `code` | `Integer` | HTTP status code (e.g. `200`, `404`). |
| `status` | `String` | Canonical status reason phrase (e.g. `"OK"`, `"Not Found"`). |
| `version` | `String` | HTTP version string (e.g. `"HTTP/1.1"`, `"HTTP/2.0"`). |
| `headers` | `Dictionary` | Response headers. Keys are header names (`String`); values are `String` (or `Octet` if non-UTF-8). |
| `url` | `String` | Final response URL after redirects. |
| `remote_addr` | `String` \| `void` | Remote socket address if available, otherwise `void`. |
| `text` | `String` | Present only when `response_type` is `"text"`. |
| `bytes` | `Octet` | Present only when `response_type` is `"bytes"`. |
| `json` | `Variant` | Present only when `response_type` is `"json"`. |

---

### `AbortSignal`

A controller/signal object for cancelling in-flight HTTP requests.

#### `new AbortSignal()`

Creates a new `AbortSignal` instance.

#### Properties

- **`aborted`** (`Boolean`, read-only): `true` if `abort()` has been called, `false` otherwise.
- **`reason`** (`Variant`, read-only): The reason value passed to `abort(reason)`, or `void` if not specified.

#### Methods

- **`abort(reason)`**: Marks the signal as aborted and cancels any in-flight requests associated with it.
  - `reason` (`Variant`, optional): Custom abort reason forwarded to `onerror`. Defaults to `"Aborted"` in `onerror` when omitted.

---

## Examples

### 1. Basic Text Request (Default)

```tjs
Plugins.link("httpreq.dll");

HttpReq.get(
    "https://example.com",
    void,
    function(res) {
        Debug.message("Status:", res.code, res.status);
        Debug.message("Body:", res.text);
    },
    function(err) {
        Debug.message("Request failed:", err);
    }
);
```

### 2. JSON Request (Omitting `onerror`)

```tjs
HttpReq.get(
    "https://lpd.lifegpc.com/api/version",
    %[ response_type: "json" ],
    function(res) {
        Debug.message("HTTP version:", res.version);
        Debug.message("API version:", res.json.version[0]);
    }
);
```

### 3. Binary (`Octet`) Request & Handling Non-2xx Statuses Manually

```tjs
HttpReq.get(
    "https://example.com/file.bin",
    %[
        response_type: "bytes",
        no_error_handle: true
    ],
    function(res) {
        if (!res.ok) {
            Debug.message("Server returned error status:", res.code, res.status);
            return;
        }
        Debug.message("Downloaded bytes length:", res.bytes.length);
    },
    function(err) {
        Debug.message("Network error:", err);
    }
);
```

### 4. Aborting an In-Flight Request

```tjs
var signal = new AbortSignal();

HttpReq.get(
    "https://httpbin.org/delay/5",
    %[
        response_type: "json",
        signal: signal
    ],
    function(res) {
        Debug.message("Completed:", res.code);
    },
    function(err) {
        // Prints: "Aborted: User cancelled the request"
        Debug.message("Aborted:", err);
    }
);

// Cancel the request with a custom reason
signal.abort("User cancelled the request");
Debug.message("Is aborted:", signal.aborted); // true
Debug.message("Reason:", signal.reason);       // "User cancelled the request"
```
