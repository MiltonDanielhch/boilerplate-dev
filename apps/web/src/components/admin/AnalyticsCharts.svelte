<script lang="ts">
	import { onMount } from "svelte";
	import { Chart, registerables } from "chart.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import { RefreshCw } from "lucide-svelte";
	import api from "$lib/api/axios";

	Chart.register(...registerables);

	let userChartCanvas: HTMLCanvasElement;
	let statusChartCanvas: HTMLCanvasElement;
	let userChart: Chart;
	let statusChart: Chart;
	
	let loading = $state(true);
	let days = $state(30);

	async function loadAnalytics() {
		loading = true;
		try {
			const res = await api.get(`/admin/analytics?days=${days}`);
			const data = res.data;

			updateCharts(data);
		} catch (err) {
			console.error("Failed to load analytics", err);
		} finally {
			loading = false;
		}
	}

	function updateCharts(data: any) {
		const labels = data.users_over_time.map((d: any) => d.label);
		
		// User Growth Chart
		if (userChart) userChart.destroy();
		userChart = new Chart(userChartCanvas, {
			type: 'line',
			data: {
				labels,
				datasets: [
					{
						label: 'New Users',
						data: data.users_over_time.map((d: any) => d.value),
						borderColor: '#3b82f6',
						backgroundColor: 'rgba(59, 130, 246, 0.1)',
						fill: true,
						tension: 0.4
					},
					{
						label: 'New Leads',
						data: data.leads_over_time.map((d: any) => d.value),
						borderColor: '#8b5cf6',
						backgroundColor: 'rgba(139, 92, 246, 0.1)',
						fill: true,
						tension: 0.4
					}
				]
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { display: true, labels: { color: '#94a3b8' } }
				},
				scales: {
					x: { grid: { display: false }, ticks: { color: '#64748b' } },
					y: { grid: { color: '#1e293b' }, ticks: { color: '#64748b' } }
				}
			}
		});

		// Leads Status Chart
		if (statusChart) statusChart.destroy();
		statusChart = new Chart(statusChartCanvas, {
			type: 'doughnut',
			data: {
				labels: data.leads_by_status.map((d: any) => d.label),
				datasets: [{
					data: data.leads_by_status.map((d: any) => d.value),
					backgroundColor: [
						'#3b82f6', '#8b5cf6', '#10b981', '#f59e0b', '#ef4444'
					],
					borderWidth: 0
				}]
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { position: 'bottom', labels: { color: '#94a3b8' } }
				}
			}
		});
	}

	onMount(() => {
		loadAnalytics();
	});
</script>

<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
	<Card.Root class="md:col-span-2 bg-slate-900 border-slate-800">
		<Card.Header class="flex flex-row items-center justify-between">
			<div>
				<Card.Title>Growth Trend</Card.Title>
				<Card.Description>Users and Leads in the last {days} days</Card.Description>
			</div>
			<div class="flex gap-2">
				<select 
					bind:value={days} 
					onchange={loadAnalytics}
					class="bg-slate-950 border-slate-800 text-xs rounded px-2 py-1 outline-none"
				>
					<option value={7}>7 Days</option>
					<option value={30}>30 Days</option>
					<option value={90}>90 Days</option>
				</select>
			</div>
		</Card.Header>
		<Card.Content class="h-[300px]">
			<canvas bind:this={userChartCanvas}></canvas>
		</Card.Content>
	</Card.Root>

	<Card.Root class="bg-slate-900 border-slate-800">
		<Card.Header>
			<Card.Title>Leads Distribution</Card.Title>
			<Card.Description>By Current Status</Card.Description>
		</Card.Header>
		<Card.Content class="h-[300px]">
			<canvas bind:this={statusChartCanvas}></canvas>
		</Card.Content>
	</Card.Root>
</div>
