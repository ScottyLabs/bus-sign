<script lang="ts">
    import { onMount } from "svelte";
    import Header from "./lib/Header.svelte";
    import Footer from "./lib/Footer.svelte";
    import BusList from "./lib/BusList.svelte";

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
        class="flex flex-1 min-h-0 flex-row items-stretch px-13.5 py-12.5 gap-11 w-full"
    >
        <BusList />
        <div class="w-1 shrink-0 bg-light-gray"></div>
        <BusList />
    </div>
    <Footer />
</main>
