import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { SecretConstruct } from './constructs/sercret-construct';
import { WebhookConstruct } from './constructs/webhook-construct';
// import * as sqs from 'aws-cdk-lib/aws-sqs';

export class CacheOffEcsLambdaStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);


    const secretConstruct = new SecretConstruct(this, 'SecretConstruct');
    const webhookConstruct = new WebhookConstruct(this, 'WebhookConstruct', {
      secret: secretConstruct.secret
    })
  }
}
