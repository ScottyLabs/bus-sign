<script lang="ts">
  import { onMount } from 'svelte';
  import BusTimeEntry from './lib/BusTimeEntry.svelte';
  import QRcode from './assets/QRcode.png';

  type ApiPrediction = {
    ETA?: string;
    location?: string;
    route?: string;
    busNumber?: string;
  };

  type StopResponse = {
    stop: string;
    predictions: ApiPrediction[];
  };

  type Entry = {
    route: string;
    location: string;
    time: string;
    busNumber: string;
  };

  let entriesUC: Entry[] = [];
  let entriesTep: Entry[] = [];

  const API_BASE = 'http://172.24.148.20:8787'; // change this later, prob to an env

  const toEntries = (predictions: ApiPrediction[]): Entry[] =>
    predictions.map((prediction) => ({
      route: prediction.route ?? '',
      location: prediction.location ?? '',
      time: prediction.ETA ?? '',
      busNumber: prediction.busNumber ?? '',
    }));

  const fetchStop = async (stop: '4407' | '7117'): Promise<Entry[]> => {
    const response = await fetch(`${API_BASE}/stop?stop=${stop}`, {
      cache: 'no-store',
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch stop ${stop}: ${response.status}`);
    }

    const data = (await response.json()) as StopResponse;
    return toEntries(data.predictions ?? []);
  };

  const refresh = async () => {
    try {
      const [uc, tep] = await Promise.all([fetchStop('7117'), fetchStop('4407')]);
      entriesUC = uc;
      entriesTep = tep;
    } catch (error) {
      console.error(error);
    }
  };

  onMount(() => {
    void refresh();
    const interval = setInterval(refresh, 10_000);
    return () => clearInterval(interval);
  });
</script>

<main>
  <div class="container" style="justify-content: start; align-items: flex-start">
    <div class="stack left">
      <div style="font-size: 40px">
        UC Side (Stop 7117)
      </div>
      {#each entriesUC as entry (entry.busNumber)}
        <BusTimeEntry {...entry} />
      {:else}
        <BusTimeEntry route={'No Buses Running'} location={''} time={''} busNumber={''}/>
      {/each}
    </div>
    <div class="stack left">
      <div style="font-size: 40px">
        Tepper Side (Stop 4407)
      </div>
      {#each entriesTep as entry (entry.busNumber)}
        <BusTimeEntry {...entry} />
      {:else}
        <BusTimeEntry route={'No Buses Running'} location={''} time={''} busNumber={''}/>
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
