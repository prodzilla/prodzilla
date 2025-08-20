import { z } from 'zod'

const envSchema = z.object({
  DATABASE_URL: z.string().url().default('postgresql://localhost:5432/end_to_end_test'),
  FIN_USER_SERVICE_BASE_URL: z.string().url(),
  AUTH_SERVICE_BASE_URL: z.string().url(),
  ACTIVATION_SERVICE_BASE_URL: z.string().url(),
  ENTITLEMENTS_V2_BASE_URL: z.string().url(),
  PORTABILITY_SERVICE_BASE_URL: z.string().url(),
})

const parseResult = envSchema.safeParse(process.env)

if (!parseResult.success) {
  throw new Error(`failed to parse environment variables:\n${parseResult.error.toString()}`)
}

const env = parseResult.data

// Services array with name and url
export const services = [
  { name: 'user', url: env.FIN_USER_SERVICE_BASE_URL },
  { name: 'auth', url: env.AUTH_SERVICE_BASE_URL },
  { name: 'activation', url: env.ACTIVATION_SERVICE_BASE_URL },
  { name: 'entitlements', url: env.ENTITLEMENTS_V2_BASE_URL },
  { name: 'portability', url: env.PORTABILITY_SERVICE_BASE_URL },
]

export default env
