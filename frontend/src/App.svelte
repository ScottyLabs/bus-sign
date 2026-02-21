<script lang="ts">
  import { onMount } from 'svelte';
  import BusTimeEntry from './lib/BusTimeEntry.svelte';
  import QRcode from './assets/QRcode.png';

  type RouteInformation = {
    route: string;
    destination: string;
    arrivals: {
      bus_id: string;
      capacity: string;
      seconds: number;
    }[];
  };

  type APIResponse = {
    [stopId: string]: RouteInformation[];
  };

  let entriesUC: RouteInformation[] = [];
  let entriesTep: RouteInformation[] = [];

  const API_HOST = import.meta.env.VITE_API_HOST || 'localhost';
  const API_PORT = import.meta.env.VITE_API_PORT || '8080';
  const API_BASE = `http://${API_HOST}:${API_PORT}`;

  const fetchPredictions = async (): Promise<APIResponse> => {
    const response = await fetch(`${API_BASE}/predictions`, {
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch predictions: ${response.status}`);
    }

    const data = (await response.json()) as APIResponse;
    return data;
  };

  const refresh = async () => {
    try {
      const data = await fetchPredictions();
      entriesUC = (data['7117'] || []).sort((a, b) =>
        (a.arrivals[0]?.seconds || Infinity) - (b.arrivals[0]?.seconds || Infinity)
      );
      entriesTep = (data['4407'] || []).sort((a, b) =>
        (a.arrivals[0]?.seconds || Infinity) - (b.arrivals[0]?.seconds || Infinity)
      );
    } catch (error) {
      console.error(error);
    }
  };

  onMount(() => {
    void refresh();
    const interval = setInterval(refresh, 3_000);
    return () => clearInterval(interval);
  });
</script>

<main>
  <div class="container" style="justify-content: start; align-items: flex-start">
    <div class="stack left">
      <div style="font-size: 40px">
        UC Side (Stop 7117)
      </div>
      {#each entriesUC as entry (entry.route + entry.destination)}
        <BusTimeEntry {...entry} />
      {:else}
        <BusTimeEntry route={'No Buses Running'} destination={''} arrivals={[]}/>
      {/each}
    </div>
    <div class="stack left">
      <div style="font-size: 40px">
        Tepper Side (Stop 4407)
      </div>
      {#each entriesTep as entry (entry.route + entry.destination)}
        <BusTimeEntry {...entry} />
      {:else}
        <BusTimeEntry route={'No Buses Running'} destination={''} arrivals={[]}/>
      {/each}
    </div>
  </div>
  <div class="footer">
    <img src={QRcode} alt="QR code" />
  </div>
</main>

<style>
  .stack {
    flex: 1 1 0;
    gap: 0.75rem;
  }
  .footer {
    background-color: black;
    height: 20%;
  }

  img {
    width: auto;
    height: 100%;
    display: block;
    margin-left: auto;
  }
</style>
