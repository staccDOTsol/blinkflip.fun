/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/chancy.json`.
 */
export type Chancy = {
  "address": "76NigLJb5MPHMz4UyYHeAKR1v4Ck1SFrAkBjVKmbYJpA",
  "metadata": {
    "name": "chancy",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "commit",
      "discriminator": [
        223,
        140,
        142,
        165,
        229,
        208,
        156,
        74
      ],
      "accounts": [
        {
          "name": "house",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  104,
                  111,
                  117,
                  115,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "dev"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "dev",
          "writable": true,
          "signer": true,
          "address": "GnTdsyRzW2pdAeWbfWwU2Uut6ghLfW6R1dsDUyDEUHUU"
        },
        {
          "name": "referral"
        },
        {
          "name": "userAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "reveal",
      "discriminator": [
        9,
        35,
        59,
        190,
        167,
        249,
        76,
        115
      ],
      "accounts": [
        {
          "name": "house",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  104,
                  111,
                  117,
                  115,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "dev"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true
        },
        {
          "name": "recentBlockhashes"
        },
        {
          "name": "dev",
          "writable": true,
          "signer": true,
          "address": "GnTdsyRzW2pdAeWbfWwU2Uut6ghLfW6R1dsDUyDEUHUU"
        },
        {
          "name": "referral",
          "writable": true
        },
        {
          "name": "userAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "house",
      "discriminator": [
        21,
        145,
        94,
        109,
        254,
        199,
        210,
        151
      ]
    },
    {
      "name": "user",
      "discriminator": [
        159,
        117,
        95,
        227,
        239,
        151,
        58,
        236
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "revealTooLate"
    },
    {
      "code": 6001,
      "name": "invalidState"
    },
    {
      "code": 6002,
      "name": "houseNotReady"
    },
    {
      "code": 6003,
      "name": "invalidUser"
    },
    {
      "code": 6004,
      "name": "invalidModulus"
    },
    {
      "code": 6005,
      "name": "invalidBlockhashes"
    }
  ],
  "types": [
    {
      "name": "gameState",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "ready"
          },
          {
            "name": "committed"
          },
          {
            "name": "revealed"
          }
        ]
      }
    },
    {
      "name": "house",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "commitSlot",
            "type": "u64"
          },
          {
            "name": "state",
            "type": {
              "defined": {
                "name": "gameState"
              }
            }
          }
        ]
      }
    },
    {
      "name": "user",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "referral",
            "type": "pubkey"
          }
        ]
      }
    }
  ]
};
