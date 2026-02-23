<script lang="ts">
    import { onMount } from "svelte";
    import BusTimeEntry from "./lib/BusTimeEntry.svelte";

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
            paddingX = Math.max(entriesUC.length, entriesTep.length) <= 6 ? 16 : 4;
            paddingY = Math.max(entriesUC.length, entriesTep.length) <= 6 ? 12 : 3;
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
    <div class="header">
        Live PRT Bus Arrivals (Forbes and Morewood)
    </div>
    <div
        class="container"
        style="justify-content: start; align-items: flex-start"
    >
        <div class="stack left">
            <div style="font-size: 40px">UC Side (Stop 7117)</div>
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
            <div style="font-size: 40px">Tepper Side (Stop 4407)</div>
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
    <footer class="footer">
        <div class="stack footer-text">
            <p class="attribution">
                Project by Undergraduate Student Senate via collaboration with ScottyLabs.<br>
                Funded in part by your Student Activities Fee.
            </p>
            <p class="disclaimer">
                Data provided under license from PRT; this application is not
                officially endorsed by Pittsburgh Regional Transit.
            </p>
        </div>
        {#if formattedTime}
            <p class="last-updated">Last updated: {formattedTime}</p>
        {/if}
    </footer>
</main>

<style>
    .stack {
        flex: 1 1 0;
        gap: 0.75rem;
    }

    .attribution {
        font-weight: bold;
        font-size: 40px;
        color: black;
        padding-left: 20px;
        margin: 0;
    }

    .disclaimer {
        font-weight: normal;
        color: gray;
        padding-left: 20px;
        margin: 0;
    }

    .footer-text {
        gap: 0;
        padding: 0.25rem 0;
    }

    .header {
        display: flex;
        justify-content: center;
        align-items: center;
        font-size: 80px;
    }

    .footer {
        position: fixed;
        bottom: 0;
        left: 0;
        width: 100%;
        background: white;
        border: 1px solid #ccc;
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
        z-index: 1000;
    }

    .last-updated {
        font-weight: normal;
        color: gray;
        font-size: 14px;
        padding-right: 20px;
    }
</style>
