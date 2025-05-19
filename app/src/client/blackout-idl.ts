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
  "errors": [
    {
      "code": 6000,
      "name": "InsufficientComputeUnits",
      "msg": "Unzureichende Compute Units"
    },
    {
      "code": 6001,
      "name": "InvalidHopIndex",
      "msg": "Ungültiger Hop-Index"
    },
    {
      "code": 6002,
      "name": "TransferNotComplete",
      "msg": "Transfer nicht abgeschlossen"
    },
    {
      "code": 6003,
      "name": "InvalidProofSize",
      "msg": "Ungültige Proof-Größe"
    }
  ]
};