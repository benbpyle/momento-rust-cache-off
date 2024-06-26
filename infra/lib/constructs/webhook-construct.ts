import { ISecret } from "aws-cdk-lib/aws-secretsmanager";
import { Construct } from "constructs";
import { RustFunction } from 'cargo-lambda-cdk';
import { FunctionUrlAuthType } from "aws-cdk-lib/aws-lambda";
import { Effect, PolicyStatement } from "aws-cdk-lib/aws-iam";

export interface WebhookConstructProps {
  secret: ISecret;
}

export class WebhookConstruct extends Construct {
  constructor(scope: Construct, id: string, props: WebhookConstructProps) {
    super(scope, id);

    let rustFunction = new RustFunction(scope, 'WebhookHandler', {
      manifestPath: 'app/lambda_webhook',
      environment: {
        RUST_LOG: "webhook_handler=debug"
      },
      memorySize: 256
    });

    rustFunction.addFunctionUrl({
      authType: FunctionUrlAuthType.NONE
    })

    rustFunction.addToRolePolicy(new PolicyStatement({
      actions: ["secretsmanager:*"],
      resources: ["*"],
      effect: Effect.ALLOW
    }))
    props.secret.grantRead(rustFunction);
  }
}
