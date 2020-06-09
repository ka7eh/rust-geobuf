import 'ol/ol.css';
import Map from 'ol/Map';
import View from 'ol/View';
import GeoJSON from 'ol/format/GeoJSON';
import { Tile as TileLayer, Vector as VectorLayer } from 'ol/layer';
import { OSM, Vector as VectorSource } from 'ol/source';

import * as geobufWasm from 'geobuf-wasm';

import countriesPBF from './countries.pbf';

// Enables error logs
geobufWasm.debug();

const geoJSONFormat = new GeoJSON({
    dataProjection: 'EPSG:4326',
    featureProjection: 'EPSG:3857'
});

const getLayer = (url) => {
    const source = new VectorSource({
        loader: (extent) => {
            const xhr = new XMLHttpRequest();
            xhr.open('GET', url);
            xhr.responseType = 'arraybuffer';
            const onError = () => {
                source.removeLoadedExtent(extent);
            };
            xhr.onerror = onError;
            xhr.onload = () => {
                if (xhr.status === 200) {
                    const geojson = geobufWasm.decode(new Uint8Array(xhr.response));
                    source.addFeatures(geoJSONFormat.readFeatures(geojson));
                } else {
                    onError();
                }
            };
            xhr.send();
        },
        useSpatialIndex: true,
        format: geoJSONFormat
    });

    return new VectorLayer({
        source,
        name
    });
};

const map = new Map({
    layers: [
        new TileLayer({
            source: new OSM()
        }),
        ...[countriesPBF].map(getLayer)
    ],
    target: 'Map',
    view: new View({
        center: [0, 0],
        zoom: 0
    })
});

map.on('click', ({ pixel }) => {
    const feature = map.forEachFeatureAtPixel(pixel, (feature) => feature);

    const info = document.getElementById('info');
    if (feature) {
        info.innerHTML = `${feature.getId()}: ${feature.get('name')}`;
    } else {
        info.innerHTML = '&nbsp;';
    }
});