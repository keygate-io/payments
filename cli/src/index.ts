#!/usr/bin/env node
import dotenv from "dotenv";
import { program } from "commander";
import chalk from "chalk";
import { proofCommand } from "./commands/proof";

async function main() {
  try {
    // Load environment variables from .env file
    dotenv.config();

    program.name("keygate").description("Keygate CLI").version("0.1.0");

    // Get proof nodes
    program
      .command("proof")
      .description("Generate merkle proof for a transaction")
      .argument("<transaction_hash>", "Transaction hash to generate proof for")
      .option("--debug", "Enable debug output")
      .action(async (transactionHash: string, options: { debug?: boolean }) => {
        try {
          await proofCommand(transactionHash, options.debug || false);
        } catch (error) {
          console.error(
            JSON.stringify(
              {
                success: false,
                error: error instanceof Error ? error.message : String(error),
              },
              null,
              2
            )
          );
          process.exit(1);
        }
      });

    await program.parseAsync();
  } catch (error) {
    console.error(chalk.red("Failed to initialize CLI:"), error);
    process.exit(1);
  }
}

if (require.main === module) {
  main().catch((error) => {
    console.error(chalk.red("Unhandled error:"), error);
    process.exit(1);
  });
}
