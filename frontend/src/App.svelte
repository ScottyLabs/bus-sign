<script lang="ts">
    import { onMount } from "svelte";
    import BusTimeEntry from "./lib/BusTimeEntry.svelte";
    import Header from "./lib/Header.svelte";
    import Footer from "./lib/Footer.svelte";

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
    let lastUpdated: Date | null = null;

    let paddingX: number = 4;
    let paddingY: number = 3;

    const API_BASE = import.meta.env.VITE_API_BASE || "";

    const fetchPredictions = async (): Promise<APIResponse> => {
        const response = await fetch(`${API_BASE}/predictions`, {
            cache: "no-store",
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
            entriesUC = (data["7117"] || []).sort(
                (a, b) =>
                    (a.arrivals[0]?.seconds || Infinity) -
                    (b.arrivals[0]?.seconds || Infinity),
            );
            entriesTep = (data["4407"] || []).sort(
                (a, b) =>
                    (a.arrivals[0]?.seconds || Infinity) -
                    (b.arrivals[0]?.seconds || Infinity),
            );
            lastUpdated = new Date();
            paddingX =
                Math.max(entriesUC.length, entriesTep.length) <= 5 ? 16 : 4;
            paddingY =
                Math.max(entriesUC.length, entriesTep.length) <= 5 ? 12 : 3;
        } catch (error) {
            console.error(error);
        }
    };

    $: formattedTime = lastUpdated
        ? lastUpdated.toLocaleTimeString("en-US", {
              hour: "numeric",
              minute: "2-digit",
              second: "2-digit",
          })
        : "";

    onMount(() => {
        void refresh();
        const interval = setInterval(refresh, 3_000);
        return () => clearInterval(interval);
    });
</script>

<main>
    <Header />
    <div
        class="container"
        style="justify-content: start; align-items: flex-start"
    >
        <div class="stack left">
            <div class="stop-header">
                UC Side <span class="arrow">&rarr;</span>
                <span class="stop-id">(Stop 7117)</span>
            </div>
            {#each entriesUC as entry (entry.route + entry.destination)}
                <BusTimeEntry {...entry} {paddingX} {paddingY} />
            {:else}
                <BusTimeEntry
                    route={"No Buses Running"}
                    destination={""}
                    arrivals={[]}
                    paddingX={16}
                    paddingY={12}
                />
            {/each}
        </div>
        <div class="stack left">
            <div class="stop-header">
                Tepper Side <span class="arrow">&larr;</span>
                <span class="stop-id">(Stop 4407)</span>
            </div>
            {#each entriesTep as entry (entry.route + entry.destination)}
                <BusTimeEntry {...entry} {paddingX} {paddingY} />
            {:else}
                <BusTimeEntry
                    route={"No Buses Running"}
                    destination={""}
                    arrivals={[]}
                    paddingX={16}
                    paddingY={12}
                />
            {/each}
        </div>
    </div>
    <Footer />
</main>

<style>
    .stack {
        flex: 1 1 0;
        gap: 0.75rem;
    }

    .stop-header {
        font-size: 40px;
        font-weight: bold;
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .stop-id {
        font-size: 30px;
        font-weight: normal;
        color: #666;
    }

    .arrow {
        font-size: 60px;
        font-weight: 900;
        color: rgba(178, 18, 196, 0.8);
        line-height: 1;
        transform: translateY(-4px);
    }
</style>
