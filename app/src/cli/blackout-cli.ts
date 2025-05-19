#!/usr/bin/env node
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { BlackoutClient } from '../client/blackout-client';
import { displayEfficiencyDashboard } from '../efficiency/terminal-dashboard';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import * as yargs from 'yargs';

async function main() {
  // Kommandozeilenargumente parsen
  const argv = yargs
    .command('transfer <amount> <recipient>', 'Führt einen anonymen Transfer durch', {
      amount: {
        description: 'Betrag in SOL',
        type: 'number',
        demandOption: true
      },
      recipient: {
        description: 'Recipient public key',
        type: 'string',
        demandOption: true
      },
      multi: {
        description: 'Additional recipient public keys (comma separated) for multi-wallet transfer',
        type: 'string',
        default: ''
      },
      efficiency: {
        description: 'Shows cost efficiency information',
        type: 'boolean',
        default: true
      }
    })
    .command('efficiency <amount>', 'Shows cost efficiency information for a transfer', {
      amount: {
        description: 'Betrag in SOL',
        type: 'number',
        demandOption: true
      },
      recipients: {
        description: 'Number of recipients (1-6)',
        type: 'number',
        default: 1
      }
    })
    .command('balance', 'Zeigt das Wallet-Guthaben an')
    .option('keypair', {
      alias: 'k',
      description: 'Pfad zur Keypair-Datei',
      type: 'string',
      default: path.join(os.homedir(), '.config', 'solana', 'id.json')
    })
    .option('url', {
      alias: 'u',
      description: 'Solana RPC URL',
      type: 'string',
      default: 'https://api.devnet.solana.com'
    })
    .option('program-id', {
      alias: 'p',
      description: 'Blackout Programm-ID',
      type: 'string',
      default: 'B1ack0utXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX'
    })
    .help()
    .alias('help', 'h')
    .argv as any;

  // Wallet aus Datei laden
  let wallet: Keypair;
  try {
    const keypairPath = argv.keypair;
    if (fs.existsSync(keypairPath)) {
      const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
      wallet = Keypair.fromSecretKey(new Uint8Array(keypairData));
      console.log(`Wallet geladen: ${wallet.publicKey.toBase58()}`);
    } else {
      console.log(`Keypair-Datei nicht gefunden: ${keypairPath}`);
      console.log('Generiere neues Wallet...');
      wallet = Keypair.generate();
      
      // Speichere das neue Wallet
      const dirPath = path.dirname(keypairPath);
      if (!fs.existsSync(dirPath)) {
        fs.mkdirSync(dirPath, { recursive: true });
      }
      fs.writeFileSync(keypairPath, JSON.stringify(Array.from(wallet.secretKey)));
      console.log(`Neues Wallet generiert und gespeichert: ${wallet.publicKey.toBase58()}`);
    }
  } catch (error) {
    console.error('Fehler beim Laden des Wallets:', error);
    wallet = Keypair.generate();
    console.log(`Temporäres Wallet generiert: ${wallet.publicKey.toBase58()}`);
  }

  // Solana-Verbindung herstellen
  const connection = new Connection(argv.url, 'confirmed');
  console.log(`Verbunden mit Solana-Netzwerk: ${argv.url}`);

  // Blackout-Programm-ID
  const programId = new PublicKey(argv['program-id']);

  // Blackout-Client initialisieren
  const blackoutClient = new BlackoutClient(connection, wallet, programId);

  // Kommando ausführen
  const command = argv._[0];
  switch (command) {
    case 'transfer':
      const amount = argv.amount * 1_000_000_000; // SOL zu Lamports
      const recipient = new PublicKey(argv.recipient);
      
      // Configure cost efficiency display
      blackoutClient.setEfficiencyInfoDisplay(Boolean(argv.efficiency));
      
      // Multi-Wallet-Transfer vorbereiten
      const additionalRecipients: PublicKey[] = [];
      if (argv.multi && typeof argv.multi === 'string' && argv.multi.trim() !== '') {
        const multiRecipients = argv.multi.split(',').map((addr: string) => addr.trim()).filter(Boolean);
        
        for (const addr of multiRecipients) {
          try {
            additionalRecipients.push(new PublicKey(addr));
          } catch (e) {
            console.warn(`Invalid recipient address skipped: ${addr}`);
          }
        }
        
        if (additionalRecipients.length > 0) {
          console.log(`Multi-wallet transfer initiated with ${additionalRecipients.length + 1} recipients`);
        }
      }
      
      console.log(`Starte anonymen Transfer: ${argv.amount} SOL an ${recipient.toBase58()}`);
      try {
        const signature = await blackoutClient.executeAnonymousTransfer(
          amount, 
          recipient,
          additionalRecipients
        );
        console.log(`Transfer abgeschlossen: ${signature}`);
      } catch (error) {
        console.error('Fehler beim Transfer:', error);
      }
      break;
      
    case 'efficiency':
      const effAmount = argv.amount * 1_000_000_000; // SOL zu Lamports
      const recipientCount = Math.min(6, Math.max(1, Number(argv.recipients || 1)));
      
      console.log(`Cost efficiency analysis for ${argv.amount} SOL with ${recipientCount} recipient${recipientCount > 1 ? 's' : ''}:`);
      
      // Verbesserte visuelle Darstellung mit dem Terminal-Dashboard
      displayEfficiencyDashboard(effAmount, recipientCount);
      break;
      
    case 'balance':
      try {
        const balance = await connection.getBalance(wallet.publicKey);
        console.log(`Wallet-Guthaben: ${balance / 1_000_000_000} SOL`);
      } catch (error) {
        console.error('Fehler beim Abrufen des Guthabens:', error);
      }
      break;
      
    default:
      console.log('Verfügbare Befehle:');
      console.log('  blackout transfer <betrag_in_sol> <empfänger> [--multi=addr1,addr2,addr3] - Führt einen anonymen Transfer durch');
      console.log('  blackout efficiency <amount_in_sol> [--recipients=n] - Shows cost efficiency information');
      console.log('  blackout balance - Zeigt das Wallet-Guthaben an');
      break;
  }
}

main().catch(console.error);