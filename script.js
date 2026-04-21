import ws from 'k6/ws';
import { check } from 'k6';

export default function () {
  const url = 'ws://localhost:3000/ws';
  const res = ws.connect(url, {}, function (socket) {
    socket.on('open', () => console.log('Conectado'));
    socket.on('message', (data) => console.log('Mensaje: ' + data));
    socket.close();
  });
}