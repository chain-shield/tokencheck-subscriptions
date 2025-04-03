# Rust Web Server

## OAuth Integration Instructions

> [!WARNING]
> Currently, only GitHub, Google and Facebook OAuth are fully implemented and tested.
> Other providers may require additional configuration or may not be fully functional.

## OAuth Integration Instructions

1. Redirect the user's browser to the `/api/auth/oauth/{provider}` endpoint on the server. Replace `{provider}` with the name of the OAuth provider (e.g., `google`, `github`).
```javascript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api';
window.location.href = `${API_BASE_URL}/auth/oauth/${provider}`;
```

2. Setup `WEB_APP_AUTH_CALLBACK_URL` variable in `.env` file in Rust server. The server will redirect to that URL after it's done with authentication.
```
WEB_APP_AUTH_CALLBACK_URL=http://localhost:3000/auth/callback
```

3. In the component that handles this authentication callback, read cookies `token` and `user`. If not present, make a call to `/api/session` which retrieves token and user data, and returns it as a JSON body in the response.
```javascript
import Cookies from 'js-cookie';
import { useEffect } from 'react';
import { useRouter } from 'next/navigation';

export default function OAuthCallbackPage() {
    const router = useRouter();
    useEffect(() => {
        async function processCallback() {
            try {
                let token = Cookies.get('token');
                let userJson = Cookies.get('user');

                if (!token || !userJson) {
                    const response = await fetch('http://localhost:8080/api/session', {
                        credentials: 'include',
                        method: 'GET'
                    });
                    if (!response.ok) {
                        throw new Error("API failed")
                    }
                    const data = await response.json();
                    token = data.token;
                    userJson = JSON.stringify(data.user);
                }

                if (!token || !userJson) {
                    // error: "Missing authentication data";
                    return;
                }

                const user = JSON.parse(userJson);
                // you can store token and user in localStorage or in a cookie here
                
                // redirect to home page or user dashboard
                router.push('/');
            } catch (error) {
                // error: ('Failed to process OAuth callback:', error);
            }
        }
        processCallback();
    }, [router]);
}
```

## Stripe Subscriptions and Payments

This application provides a complete subscription management system with Stripe integration. Below are details on how to use the subscription and payment endpoints.

### Environment Variables

- `STRIPE_SECRET_KEY`: Your Stripe API secret key
  - Example: `sk_test_51HGXXXXXXXXXXXXXxli5XXXXXXXXXXXXXXXXXXXXXXXX`
- `STRIPE_WEBHOOK_SECRET`: Your Stripe webhook signing secret
  - Example: `whsec_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXxx`

### Subscription Management Endpoints

#### Get Available Subscription Plans
- **Endpoint**: `GET /api/secured/sub/plans`
- **Authentication**: Requires JWT token
- **Response**: JSON array of available subscription plans
- **Example**:
  ```javascript
  const response = await fetch('/api/secured/sub/plans', {
    headers: { 'Authorization': `Bearer ${token}` }
  });
  const data = await response.json();
  // data.plans contains available subscription plans
  ```

#### Create Subscription
- **Endpoint**: `POST /api/secured/sub/subscribe`
- **Authentication**: Requires JWT token
- **Request Body**:
  ```json
  {
    "price_id": "price_1234567890",
    "success_url": "https://yourapp.com/subscription/success",
    "cancel_url": "https://yourapp.com/subscription/canceled"
  }
  ```
- **Response**: JSON object with Stripe checkout URL
- **Example**:
  ```javascript
  const response = await fetch('/api/secured/sub/subscribe', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      price_id: "price_1234567890",
      success_url: "https://yourapp.com/subscription/success",
      cancel_url: "https://yourapp.com/subscription/canceled"
    })
  });
  const data = await response.json();
  window.location.href = data.url; // Redirect to Stripe checkout
  ```

#### Create Enterprise Subscription
- **Endpoint**: `POST /api/secured/sub/enterprise`
- **Authentication**: Requires JWT token
- **Request Body**:
  ```json
  {
    "name": "Enterprise Plan - Custom",
    "amount": 99900,
    "interval": "month",
    "success_url": "https://yourapp.com/subscription/success",
    "cancel_url": "https://yourapp.com/subscription/canceled"
  }
  ```
- **Response**: JSON object with Stripe checkout URL

#### Get Current Subscription
- **Endpoint**: `GET /api/secured/sub/current`
- **Authentication**: Requires JWT token
- **Response**: JSON object with current subscription details or 404 if no active subscription
- **Example**:
  ```javascript
  const response = await fetch('/api/secured/sub/current', {
    headers: { 'Authorization': `Bearer ${token}` }
  });
  if (response.ok) {
    const data = await response.json();
    // data.subscription contains subscription details
  }
  ```

#### Update Auto-Renewal
- **Endpoint**: `POST /api/secured/sub/auto-renew`
- **Authentication**: Requires JWT token
- **Request Body**:
  ```json
  {
    "auto_renew": true
  }
  ```
- **Response**: JSON object with updated subscription details

### Payment Management Endpoints

#### Process Refunds
- **Endpoint**: `POST /api/secured/pay/refund`
- **Authentication**: Requires JWT token
- **Request Body**:
  ```json
  {
    "payment_intent_id": "pi_1234567890",
    "amount": 1000,
    "reason": "requested_by_customer"
  }
  ```
- **Response**: JSON object with refund details
- **Example**:
  ```javascript
  const response = await fetch('/api/secured/pay/refund', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      payment_intent_id: "pi_1234567890",
      amount: 1000, // Optional: refund $10.00
      reason: "requested_by_customer" // Optional
    })
  });
  const refundData = await response.json();
  ```

#### Get Subscription Payment Details
- **Endpoint**: `GET /api/secured/pay/subscription-payment/{subscription_id}`
- **Authentication**: Requires JWT token
- **Response**: JSON object with payment details including payment intent ID

#### Get Payment Intents
- **Endpoint**: `POST /api/secured/pay/payment-intents`
- **Authentication**: Requires JWT token
- **Request Body**:
  ```json
  {
    "user_id": "cus_1234567890",  // Optional: Default is authenticated user
    "limit": 10,                  // Optional: Default is 25
    "starting_after": "pi_1234",  // Optional: For pagination
    "ending_before": "pi_5678"    // Optional: For pagination
  }
  ```
- **Response**: JSON object with payment intents
- **Example**:
  ```javascript
  const response = await fetch('/api/secured/pay/payment-intents', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      limit: 10,
      // Leave user_id empty to get the current user's payment intents
    })
  });
  const data = await response.json();
  // data.intents contains payment intent objects
  ```

#### Webhook Handler
- **Endpoint**: `POST /api/pay/webhook`
- **No Authentication**: This endpoint is called by Stripe
- **How to Set Up**:
  1. Go to Stripe Dashboard → Developers → Webhooks
  2. Add Endpoint: https://yourapp.com/api/pay/webhook
  3. Select events to listen for (payment_intent.succeeded, etc.)
  4. Get the webhook signing secret and set it as STRIPE_WEBHOOK_SECRET

### Subscription Flow

1. **Browse Plans**: Use `/api/secured/sub/plans` to display available plans
2. **Subscribe**: Call `/api/secured/sub/subscribe` with selected plan ID
3. **Redirect to Stripe**: User completes payment on Stripe checkout page
4. **Handle Success/Cancel**: Stripe redirects to your success/cancel URL
5. **Verify Subscription**: Call `/api/secured/sub/current` to check subscription status
6. **Manage Subscription**: Update auto-renewal or handle refunds as needed

### Testing Stripe Integration

For testing, use Stripe test keys and test card numbers:
- Test Card Number: `4242 4242 4242 4242`
- Expiration: Any future date
- CVC: Any 3 digits
- ZIP: Any 5 digits