const endpointsList = document.querySelector('#endpoints');
const endpointInfo = document.querySelector('#info');
var selectedEndpoint = null;

function ip2id(ip) {
	return ip.replaceAll('.', '_').replaceAll(':', '_');
}

function buildEndpointInfo(endpoint) {
	var html = '';

	if (endpoint.name) {
		html += `<h1>${endpoint.address}
							<span class="blockquote-footer">
								${endpoint.name}
							</span>
						</h1>`;
	} else {
		html += `<h1>
							${endpoint.address}
						</h1>`;
	}

	if (endpoint.local) {
		html += `<span class="badge text-bg-warning">local</span> `;
	}

	if (endpoint.fingerprint != null) {
		if (endpoint.fingerprint.vendor) {
			html += `<span class="badge text-bg-primary">${endpoint.fingerprint.vendor}</span> `;
		}
		if (endpoint.fingerprint.kind) {
			html += `<span class="badge text-bg-secondary">${endpoint.fingerprint.kind}</span>`;
		}
	}

	html += '<hr/>';

	for (var name in endpoint.services) {
		var svc = endpoint.services[name];

		if (svc.description) {
			html += `<b>${svc.name}</b>
							<span class="blockquote-footer">
								${svc.description}
							</span>
							<br/>`;
		} else {
			html += `<b>${svc.name}</b>
							<br/>`;
		}

		html += '<ul class="list-unstyled">';

		for (var pname in svc.properties) {
			var values = svc.properties[pname];
			var badge = 'text-bg-secondary';

			if (pname == 'ipv4' || pname == 'ipv6') {
				badge = 'text-bg-success';
			} else if (pname == 'server') {
				badge = 'text-bg-warning';
			} else if (pname == 'name') {
				badge = 'text-bg-info';
			}

			html += `<li>
								&nbsp;&nbsp;&nbsp; <span class="badge rounded-pill ${badge}">${pname}</span>
								<code>${values.join(', ')}</code>
							 </li>`;
		}

		html += '</ul>';
	}

	return html;
}

function endpointIcon(endpoint) {
	if (endpoint.local) {
		return '<i class="bi bi-ethernet"></i>';
	}

	var fingerprint = endpoint.fingerprint;
	if (fingerprint != null) {
		var kind = fingerprint.kind;
		if (kind == 'chromecast') {
			return '<i class="bi bi-cast"></i>';
		} else if (kind == 'disk') {
			return '<i class="bi bi-hdd-network"></i>';
		} else if (kind == 'light') {
			return '<i class="bi bi-lightbulb"></i>';
		} else if (kind == 'osx' || kind == 'apple') {
			return '<i class="bi bi-apple"></i>';
		}
	}

	return '<i class="bi bi-pc-display"></i>';
}

function updateState(state) {
	var html = '';

	var ips = Object.keys(state);

	if (ips.length == 0) {
		return;
	}

	ips.sort((a, b) => {
		const num1 = Number(a.split(".").map((num) => (`000${num}`).slice(-3)).join(""));
		const num2 = Number(b.split(".").map((num) => (`000${num}`).slice(-3)).join(""));
		return num1 - num2;
	});

	for (var i = 0; i < ips.length; i++) {
		var ip = ips[i];
		var endpoint = state[ip];
		var active = selectedEndpoint == ip || selectedEndpoint == null && i == 0 ? ' active' : '';
		var icon = endpointIcon(endpoint);

		html += `<li class="nav-item">
							<button class="nav-link ${active}" aria-current="page" id="btn_${ip2id(ip)}">
								${icon} ${endpoint.address}
							</button>
						</li>`;
	}

	endpointsList.innerHTML = html;

	for (var ip in state) {
		var btn = document.querySelector(`#btn_${ip2id(ip)}`);
		var endpoint = state[ip];
		// capture btn and endpoint scope
		(function (btn, endpoint) {
			btn.addEventListener('click', () => {
				const active = document.querySelector('.active');
				if (active) {
					active.classList.remove('active');
				}

				btn.classList.add("active");
				selectedEndpoint = endpoint.address;
				endpointInfo.innerHTML = buildEndpointInfo(endpoint);
			});
		})(btn, endpoint);
	}

	if (selectedEndpoint == null) {
		selectedEndpoint = ips[0];
		endpointInfo.innerHTML = buildEndpointInfo(state[ips[0]]);
	} else if (selectedEndpoint in state) {
		// redraw in case there are more info
		endpointInfo.innerHTML = buildEndpointInfo(state[selectedEndpoint]);
	}
}

setInterval(function () {
	window.__TAURI__
		.invoke('get_state')
		.then(updateState)
		.catch(updateState)
}, 1000);