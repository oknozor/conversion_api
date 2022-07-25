# Conversion API [![CI](https://github.com/oknozor/conversion_api/actions/workflows/CI.yaml/badge.svg)](https://github.com/oknozor/conversion_api/actions/workflows/CI.yaml)

A REST API exposing a single endpoint to convert between weight units.
This was done as part of a technical assessment for Climate Seed.

## Howto 

**1. Run:**
`cargo run`

**2. Test:** 
`cargo test`


## Routes

The API expose a single route: `/convert` expecting the following json body : 

```json
{
  "from": "Unit",
  "to": "Unit",
  "quantity": "Number"
}
```

`Unit`: one of "gram", "kilo", "ton" or "lb". 

### Example: 

1. Start the rocket api: `cargo run`
2. Perform a conversion request:
    ```sh
    curl --request POST \
      --url http://127.0.0.1:8000/convert \
      --header 'Content-Type: application/json' \
      --data '{
      "from": "gram",
      "to": "lb",
      "quantity": 10000
    }'
    ```