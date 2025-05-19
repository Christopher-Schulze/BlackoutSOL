export const IDL = {
  "version": "0.1.0",
  "name": "blackout",
  "instructions": [
    {
      "name": "initializeTransfer",
      "accounts": [
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "transferState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "proofData",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "executeHop",
      "accounts": [
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "transferState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "hopIndex",
          "type": "u8"
        },
        {
          "name": "proofData",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "finalizeTransfer",
      "accounts": [
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "transferState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "recipient",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "proofData",
          "type": "bytes"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "TransferState",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "currentHop",
            "type": "u8"
          },
          {
            "name": "seed",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "closed",
            "type": "bool"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "TransferInitialized",
      "fields": [
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount",
          "type": "u64",
          "index": false
        },
        {
          "name": "seed",
          "type": {
            "array": [
              "u8",
              32
            ]
          },
          "index": false
        }
      ]
    },
    {
      "name": "HopExecuted",
      "fields": [
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "hopIndex",
          "type": "u8",
          "index": false
        }
      ]
    },
    {
      "name": "TransferFinalized",
      "fields": [
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "recipient",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "InsufficientComputeUnits",
      "msg": "Insufficient compute units"
    },
    {
      "code": 6001,
      "name": "InvalidHopIndex",
      "msg": "Invalid hop index"
    },
    {
      "code": 6002,
      "name": "TransferNotComplete",
      "msg": "Transfer not complete"
    },
    {
      "code": 6003,
      "name": "InvalidProofSize",
      "msg": "Invalid proof size"
    }
  ]
};