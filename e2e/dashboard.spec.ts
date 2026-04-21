import { test, expect } from '@playwright/test';

test('Debe mostrar el pedido de Santiago Urdaneta en el dashboard', async ({ page, request }) => {
    // 1. Ir al Dashboard
    await page.goto('http://localhost:3000');
    
    // 2. Verificar que el sistema esté online
    const status = page.locator('#status');
    await expect(status).toContainText('SISTEMA ONLINE');

    // 3. DISPARAR EL PEDIDO (Simulación)
    // Esto activa la lógica en Rust, el semáforo y el broadcast
    await request.get('http://localhost:3000/simular');

    // 4. Verificar que aparezca la tarjeta
    const orderCard = page.locator('.order-card').filter({ hasText: 'Santiago Urdaneta' });
    await expect(orderCard).toBeVisible({ timeout: 10000 });
    await expect(orderCard).toContainText('$');
});