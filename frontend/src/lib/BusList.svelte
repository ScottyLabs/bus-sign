<script lang="ts">
    import type { Component } from "svelte";
    import { flip } from "svelte/animate";
    import { slide } from "svelte/transition";
    import ArrowLeft from "~icons/lucide/arrow-left";
    import ArrowRight from "~icons/lucide/arrow-right";
    import BusTimeEntry from "./BusTimeEntry.svelte";

    type RouteInformation = {
        route: string;
        destination: string;
        arrivals: {
            bus_id: string;
            capacity: string;
            seconds: number;
        }[];
    };

    // type MajorRoute = { route: string; minutes: number };

    export let title: string;
    export let direction: "inbound" | "outbound";
    export let stopId: string;
    export let walkMins: number;
    export let near: string;
    export let entries: RouteInformation[];

    // /** How many arrivals fit in the main column before overflow goes to major routes. */
    // const MAIN_DISPLAY_LIMIT = 6;
    // const PINNED_MAJOR_ROUTE = "28X";
    // const MAJOR_ROTATE_MS = 7_000;

    const arrows: Record<"inbound" | "outbound", Component> = {
        inbound: ArrowLeft,
        outbound: ArrowRight,
    };

    // let rotateIndex = 0;

    // const toMajorRoute = (entry: RouteInformation): MajorRoute => ({
    //     route: entry.route,
    //     minutes: Math.max(
    //         1,
    //         Math.ceil((entry.arrivals[0]?.seconds ?? 0) / 60),
    //     ),
    // });

    $: Arrow = arrows[direction];
    // $: mainEntries = entries.slice(0, MAIN_DISPLAY_LIMIT);
    // $: overflowEntries = entries.slice(MAIN_DISPLAY_LIMIT);

    // $: pinnedMajor = (() => {
    //     const entry = entries.find((e) => e.route === PINNED_MAJOR_ROUTE);
    //     return entry ? toMajorRoute(entry) : null;
    // })();

    // $: rotatePool = overflowEntries.filter(
    //     (e) => e.route !== PINNED_MAJOR_ROUTE,
    // );

    // $: rotatingMajor =
    //     rotatePool.length > 0
    //         ? toMajorRoute(rotatePool[rotateIndex % rotatePool.length])
    //         : null;

    // $: nextMajorRoutes = [pinnedMajor, rotatingMajor].filter(
    //     (r): r is MajorRoute => r != null,
    // );

    // onMount(() => {
    //     const interval = setInterval(() => {
    //         if (rotatePool.length === 0) return;
    //         rotateIndex = (rotateIndex + 1) % rotatePool.length;
    //     }, MAJOR_ROTATE_MS);
    //     return () => clearInterval(interval);
    // });
</script>

<div class="flex flex-1 min-w-0 flex-col gap-9">
    <div class="self-stretch flex flex-col justify-start items-start gap-2.5">
        <div class="self-stretch inline-flex justify-between items-start">
            <div class="inline-flex flex-col justify-start items-start">
                <div class="inline-flex justify-start items-center gap-3">
                    <div
                        class="justify-start text-black text-5xl font-bold leading-[56px]"
                    >
                        {title}
                    </div>
                    <Arrow class="size-9 shrink-0" />
                </div>
                <div class="flex justify-start items-center gap-2">
                    <div
                        class="justify-start text-gray text-xl font-semibold leading-7"
                    >
                        Stop #{stopId}
                    </div>
                    <div class="w-[3px] h-7 shrink-0 bg-light-gray"></div>
                    <div class="justify-start text-xl font-semibold leading-7">
                        <span class="text-gray"
                            >{walkMins} mins walk near
                        </span>{" "}<span class="text-black">{near}</span>
                    </div>
                </div>
            </div>
            <!--
            <div class="inline-flex flex-col justify-start items-end gap-1">
                <div class="justify-start text-black text-xl font-semibold">
                    Next major routes
                </div>
                <div
                    class="text-right justify-start text-gray text-xl font-medium leading-7"
                >
                    {#each nextMajorRoutes as major, i}
                        {#if i > 0}<br />{/if}<span class="text-black"
                            >{major.route}</span
                        > in {major.minutes} min
                    {/each}
                </div>
            </div>
            -->
        </div>
        <div class="self-stretch h-1 bg-red"></div>
    </div>
    <div
        class="self-stretch flex flex-col justify-start items-start gap-5 overflow-hidden min-h-0"
    >
        {#each entries as entry (`${entry.route}:${entry.destination}`)}
            <div
                class="self-stretch"
                animate:flip={{ duration: 400 }}
                transition:slide={{ duration: 300 }}
            >
                <BusTimeEntry
                    route={entry.route}
                    destination={entry.destination}
                    arrivals={entry.arrivals}
                />
            </div>
        {/each}
    </div>
</div>
