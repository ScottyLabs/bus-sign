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
    <div
        class="container"
        style="justify-content: start; align-items: flex-start"
    >
        <div class="stack left">
            <div style="font-size: 40px">UC Side (Stop 7117)</div>
            {#each entriesUC as entry (entry.route + entry.destination)}
                <BusTimeEntry {...entry} />
            {:else}
                <BusTimeEntry
                    route={"No Buses Running"}
                    destination={""}
                    arrivals={[]}
                />
            {/each}
        </div>
        <div class="stack left">
            <div style="font-size: 40px">Tepper Side (Stop 4407)</div>
            {#each entriesTep as entry (entry.route + entry.destination)}
                <BusTimeEntry {...entry} />
            {:else}
                <BusTimeEntry
                    route={"No Buses Running"}
                    destination={""}
                    arrivals={[]}
                />
            {/each}
        </div>
    </div>
    <p class="disclaimer">
        Data provided under license from PRT; this application is not officially
        endorsed by Pittsburgh Regional Transit.
    </p>
</main>

<style>
    .stack {
        flex: 1 1 0;
        gap: 0.75rem;
    }

    .disclaimer {
        font-weight: normal;
        color: gray;
        padding-left: 20px;
    }
</style>
