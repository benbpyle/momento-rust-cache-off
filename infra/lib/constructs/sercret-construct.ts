import { ISecret, Secret } from "aws-cdk-lib/aws-secretsmanager";
import { Construct } from "constructs";

export class SecretConstruct extends Construct {
  private _secret: ISecret;


  public get secret(): ISecret {
    return this._secret;
  }

  constructor(scope: Construct, id: string) {
    super(scope, id);

    this._secret = Secret.fromSecretCompleteArn(scope, 'MomentoSecret', 'arn:aws:secretsmanager:us-west-2:252703795646:secret:moment-webhook-token-brOeW3');
  }
}
