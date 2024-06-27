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
        },
        {
          "name": "lookupTableTable",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  111,
                  111,
                  107,
                  117,
                  112,
                  95,
                  116,
                  97,
                  98,
                  108,
                  101,
                  95,
                  116,
                  97,
                  98,
                  108,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "refCount",
          "type": "u8"
        },
        {
          "name": "lutCount",
          "type": "u8"
        }
      ]
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
      "name": "lookupTableTable",
      "discriminator": [
        196,
        217,
        63,
        239,
        92,
        90,
        20,
        219
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
            "name": "recentWinner",
            "type": "pubkey"
          },
          {
            "name": "recentReferrer",
            "type": "pubkey"
          },
          {
            "name": "recentWon",
            "type": "u64"
          },
          {
            "name": "recentReferrerWon",
            "type": "u64"
          },
          {
            "name": "recentReferralChain",
            "type": "u8"
          },
          {
            "name": "totalWins",
            "type": "u16"
          },
          {
            "name": "totalWon",
            "type": "u64"
          },
          {
            "name": "totalInflow",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "lookupTableTable",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "lookupTables",
            "type": {
              "vec": "pubkey"
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
          },
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "streak",
            "type": "u64"
          },
          {
            "name": "state",
            "type": {
              "defined": {
                "name": "gameState"
              }
            }
          },
          {
            "name": "lastPlay",
            "type": "i64"
          },
          {
            "name": "totalAmount",
            "type": "u64"
          }
        ]
      }
    }
  ]
};
